import { appendFileSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import { spawnSync } from 'node:child_process';

const runDirectory = process.argv[2];
if (!runDirectory) {
  throw new Error('health-only observer requires a run directory');
}
mkdirSync(runDirectory, { recursive: true });

const hostPath = `${runDirectory}/host.csv`;
const processPath = `${runDirectory}/processes.csv`;
const apiPidPath = `${runDirectory}/api.pid`;
const k6PidPath = `${runDirectory}/k6.pid`;
const intervalMs = Number(process.env.XFLY_OBSERVE_INTERVAL_MS || 5000);
const apiPort = process.env.XFLY_API_PORT || '18080';

writeFileSync(hostPath, 'timestamp,cpu_percent,load1,load5,load15,mem_used_bytes,mem_total_bytes\n');
writeFileSync(processPath, 'timestamp,api_pid,api_fd_count,api_status,k6_pid,k6_fd_count,k6_status\n');

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

function timestamp() {
  return new Date().toISOString();
}

function validPid(value) {
  return /^\d+$/.test(String(value || '')) ? String(value) : '';
}

function readPidFile(path) {
  try {
    return validPid(readFileSync(path, 'utf8').trim());
  } catch {
    return '';
  }
}

function listenerPid() {
  const result = spawnSync(
    'lsof',
    ['-nP', '-a', `-tiTCP:${apiPort}`, '-sTCP:LISTEN'],
    { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] },
  );
  if (result.status !== 0) return '';
  return validPid(result.stdout.split(/\s+/).find(Boolean));
}

function apiPid() {
  return validPid(process.env.XFLY_API_PID) || readPidFile(apiPidPath) || listenerPid();
}

function isRunning(pid) {
  if (!pid) return false;
  try {
    process.kill(Number(pid), 0);
    return true;
  } catch {
    return false;
  }
}

function descriptorCount(pid) {
  const result = spawnSync(
    'lsof',
    ['-nP', '-p', pid],
    { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] },
  );
  if (result.status !== 0) return '';
  const lines = result.stdout.trim() ? result.stdout.trim().split('\n') : [];
  return String(Math.max(lines.length - 1, 0));
}

function processObservation(pid) {
  if (!pid) return { pid: '', fdCount: '', status: 'not_found' };
  if (!isRunning(pid)) return { pid, fdCount: '', status: 'not_running' };
  const fdCount = descriptorCount(pid);
  return {
    pid,
    fdCount,
    status: fdCount === '' ? 'fd_count_unavailable' : 'ok',
  };
}

function sample(previousCpu) {
  const now = timestamp();
  const currentCpu = cpuSnapshot();
  const loads = os.loadavg();
  const totalMemory = os.totalmem();
  const freeMemory = os.freemem();
  appendFileSync(
    hostPath,
    `${now},${cpuPercent(previousCpu, currentCpu)},${loads[0].toFixed(2)},${loads[1].toFixed(2)},${loads[2].toFixed(2)},${totalMemory - freeMemory},${totalMemory}\n`,
  );

  const api = processObservation(apiPid());
  const k6 = processObservation(readPidFile(k6PidPath));
  appendFileSync(
    processPath,
    `${now},${api.pid},${api.fdCount},${api.status},${k6.pid},${k6.fdCount},${k6.status}\n`,
  );
  return currentCpu;
}

let stopping = false;
let previousCpu = sample();
const timer = setInterval(() => {
  if (!stopping) previousCpu = sample(previousCpu);
}, intervalMs);

function stop() {
  if (stopping) return;
  stopping = true;
  clearInterval(timer);
}

process.on('SIGTERM', stop);
process.on('SIGINT', stop);
