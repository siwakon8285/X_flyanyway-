import { appendFileSync, mkdirSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import { spawnSync } from 'node:child_process';

const runDirectory = process.argv[2];
if (!runDirectory) {
  throw new Error('observer requires a run directory');
}
mkdirSync(runDirectory, { recursive: true });

const hostPath = `${runDirectory}/host.csv`;
const postgresPath = `${runDirectory}/postgres.csv`;
const intervalMs = Number(process.env.XFLY_OBSERVE_INTERVAL_MS || 5000);
const postgresQuery = `
SELECT current_database(),
       current_setting('max_connections'),
       COUNT(*),
       COUNT(*) FILTER (WHERE state = 'active'),
       COUNT(*) FILTER (WHERE wait_event IS NOT NULL AND state <> 'idle'),
       COUNT(*) FILTER (WHERE wait_event_type = 'Lock')
FROM pg_stat_activity;
`;

writeFileSync(hostPath, 'timestamp,cpu_percent,load1,mem_used_bytes,mem_total_bytes\n');
writeFileSync(postgresPath, 'timestamp,total_connections,active_connections,waiting_sessions,blocked_sessions,max_connections,observer_status\n');

function cpuSnapshot() {
  return os.cpus().reduce((total, cpu) => {
    const values = cpu.times;
    return {
      idle: total.idle + values.idle,
      total: total.total + values.user + values.nice + values.sys + values.irq + values.idle,
    };
  }, { idle: 0, total: 0 });
}

function cpuPercent(previous, current) {
  if (!previous) return '';
  const totalDelta = current.total - previous.total;
  const idleDelta = current.idle - previous.idle;
  if (totalDelta <= 0) return '';
  return ((1 - (idleDelta / totalDelta)) * 100).toFixed(2);
}

function pgEnvironment(databaseUrl) {
  let parsed;
  try {
    parsed = new URL(databaseUrl);
  } catch {
    return null;
  }
  if (!['postgres:', 'postgresql:'].includes(parsed.protocol)) return null;
  const database = decodeURIComponent(parsed.pathname.replace(/^\//, ''));
  if (!database) return null;
  const environment = {
    ...process.env,
    PGHOST: parsed.hostname,
    PGPORT: parsed.port || '5432',
    PGDATABASE: database,
    PGAPPNAME: 'x-fly-phase1-observer',
  };
  if (parsed.username) environment.PGUSER = decodeURIComponent(parsed.username);
  if (parsed.password) environment.PGPASSWORD = decodeURIComponent(parsed.password);
  return environment;
}

function postgresSample() {
  const environment = pgEnvironment(process.env.XFLY_TEST_DATABASE_URL || '');
  if (!environment) return { status: 'invalid_observer_configuration' };
  const result = spawnSync(
    'psql',
    ['--no-psqlrc', '--quiet', '--tuples-only', '--no-align', '--field-separator', '\t', '--command', postgresQuery],
    { env: environment, encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] },
  );
  if (result.status !== 0) return { status: 'unavailable' };
  const fields = result.stdout.trim().split('\t').map((value) => value.trim());
  if (fields.length !== 6 || fields[0] !== 'x_fly_concurrency_test') {
    return { status: 'wrong_database_or_visibility' };
  }
  return {
    status: 'ok',
    total: fields[2],
    active: fields[3],
    waiting: fields[4],
    blocked: fields[5],
    maxConnections: fields[1],
  };
}

function timestamp() {
  return new Date().toISOString();
}

function sample(previousCpu) {
  const now = timestamp();
  const currentCpu = cpuSnapshot();
  const totalMemory = os.totalmem();
  const freeMemory = os.freemem();
  appendFileSync(
    hostPath,
    `${now},${cpuPercent(previousCpu, currentCpu)},${os.loadavg()[0].toFixed(2)},${totalMemory - freeMemory},${totalMemory}\n`,
  );

  const postgres = postgresSample();
  appendFileSync(
    postgresPath,
    `${now},${postgres.total ?? ''},${postgres.active ?? ''},${postgres.waiting ?? ''},${postgres.blocked ?? ''},${postgres.maxConnections ?? ''},${postgres.status}\n`,
  );
  return currentCpu;
}

let stopping = false;
process.on('SIGTERM', () => { stopping = true; });
process.on('SIGINT', () => { stopping = true; });

let previousCpu;
while (!stopping) {
  await new Promise((resolve) => setTimeout(resolve, intervalMs));
  previousCpu = sample(previousCpu);
}
