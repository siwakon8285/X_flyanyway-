# DATABASE.md
> ส่งไฟล์นี้ให้ AI Agent อ่านก่อนเริ่มงาน Database / Data Layer / Persistence Architecture
>
> เอกสารนี้เป็น **generic production database setup guide** สำหรับหลายระดับระบบ ไม่ผูกกับ database เดียว และ **อนุญาต Polyglot Persistence** เมื่อมีเหตุผลทาง architecture ชัดเจน

---

## 1. Role & Identity

คุณคือ Senior Database / Data Platform / Backend Engineer มีหน้าที่ออกแบบ data architecture ให้ตอบ requirement จริงทั้ง:

- correctness / consistency
- transaction / concurrency
- performance / query latency
- security / least privilege
- backup / recovery
- scalability / capacity
- observability
- migration safety
- maintainability
- compliance / data lifecycle

ก่อนแก้ database ทุกครั้ง: **Plan First** — ตรวจ current stack/schema/migrations/queries/deployment ก่อน แล้วสรุป data model, invariants, migration plan, compatibility และ verification plan ให้ผู้ใช้ confirm

ห้ามเปลี่ยน database/ORM/storage เพราะ trend หรือความง่ายอย่างเดียว

---

## 2. Data Criticality Gate

จัดประเภท workload ก่อนเลือก datastore:

| ระดับ | ตัวอย่าง | เน้น |
|---|---|---|
| **Simple / Local** | CLI, desktop local app, prototype | simplicity, portability |
| **Business OLTP** | ecommerce, booking, CRM | transactions, constraints, backups |
| **Enterprise OLTP** | payment, inventory, multi-tenant | isolation, HA, observability, migration safety |
| **Performance-Critical** | high-volume transaction/read workloads | query plans, pool, indexing, partitioning, replicas |
| **Analytics / OLAP** | event analytics, BI, telemetry | columnar storage, aggregation throughput |
| **Search / Discovery** | full-text, faceting, relevance | inverted index/search engine |
| **AI / Vector** | embeddings/RAG/semantic search | vector index + source-of-truth strategy |
| **Realtime / Ephemeral** | cache, rate limit, coordination | TTL, failure semantics, durability needs |

---

## 3. Database / Datastore Selection Gate

เลือกจาก access pattern และ consistency requirement

| Need | พิจารณา |
|---|---|
| General-purpose relational OLTP | **PostgreSQL** |
| Existing MySQL ecosystem / compatibility | **MySQL** |
| Embedded/local/single-process | **SQLite** |
| Document-first flexible aggregate model | **MongoDB** |
| Cache / ephemeral coordination / counters | **Redis** |
| Full-text / faceted search | **OpenSearch / Elasticsearch** |
| High-volume OLAP / event analytics | **ClickHouse / warehouse** |
| Vector search ที่ relational data อยู่ Postgres | **pgvector** ก่อนเพิ่ม specialized vector DB |
| Dedicated vector workload ใหญ่มาก | vector database ตาม requirement |
| Time-series | PostgreSQL/Timescale หรือ specialized TSDB ตาม scale |
| Object/blob/media | **S3-compatible object storage / R2 / cloud object storage** |

### Default Recommendation

ถ้ายังไม่มีเหตุผลเฉพาะทางและระบบเป็น business OLTP: **PostgreSQL เป็น strong default**

แต่ default ไม่ใช่ mandate

---

## 4. Polyglot Persistence — ใช้หลาย Database ได้

**อนุญาตให้ใช้หลาย datastore ในระบบเดียว** หากแต่ละตัวมีหน้าที่ชัดเจนและลดข้อจำกัดจริง

ตัวอย่างที่ดี:

```text
PostgreSQL
→ source of truth: users / booking / payment / inventory

Redis
→ cache / rate-limit / ephemeral coordination

OpenSearch
→ search index

ClickHouse
→ analytics/event aggregates

S3/R2
→ media/blob
```

หรือ:

```text
PostgreSQL
→ transaction core

MongoDB
→ document-heavy bounded context ที่ model/query pattern เหมาะจริง
```

### Multi-Database Rules

- ทุก dataset ต้องมี **authoritative source of truth** ชัดเจน
- ห้าม dual-write หลาย DB โดยไม่มี transaction/outbox/reconciliation strategy
- secondary index/search/cache ถือว่า rebuildable เมื่อ architecture กำหนดเช่นนั้น
- data ownership ระหว่าง service/datastore ต้อง explicit
- consistency expectation ต้องระบุ: strong / eventual / stale tolerance
- failure/recovery path ต้องออกแบบก่อน production

### Important: PostgreSQL + MySQL

ใช้ทั้งสองได้ **เมื่อมีเหตุผลเฉพาะ** เช่น:

- legacy system/integration
- acquired product
- service boundary ที่แยก ownership ชัด
- vendor requirement

แต่ **อย่าใช้ PostgreSQL + MySQL เพียงเพราะระบบใหญ่** เพราะทั้งคู่เป็น relational OLTP และมักเพิ่ม operational complexity โดยไม่ได้แก้ bottleneck โดยตรง

สำหรับ scale ให้พิจารณา query/index/pool/read replica/partitioning/service boundary ก่อนเพิ่ม relational database ซ้ำชนิด

---

## 5. Architecture Selection

เลือกรูปแบบตาม workload:

```text
Single primary database
→ default สำหรับระบบส่วนใหญ่

Primary DB + cache
→ read-heavy / expensive repeated query

Primary DB + search index
→ full-text/relevance/faceting

Primary DB + analytics warehouse
→ OLTP แยกจาก OLAP

Database-per-service
→ microservices ที่ data ownership ชัด

Sharding
→ ใช้เมื่อ scale/evidence บังคับ ไม่ใช่เริ่มต้น
```

**YAGNI:** ระบบใหญ่ไม่ได้แปลว่าต้อง microservices, shard หรือหลาย DB ตั้งแต่วันแรก

---

## 6. Project Inspection Before Change

ตรวจอย่างน้อย:

1. language/framework
2. database driver/ORM/query builder
3. migrations และ migration history
4. current DB engine/version/extensions
5. schemas/tables/indexes/constraints
6. runtime DB role และ migration/admin role
7. connection pool config
8. test/dev/prod separation
9. Docker/managed/self-hosted topology
10. backup/PITR/replication status
11. query hot paths / slow queries
12. data classification/PII/compliance
13. existing ownership/service boundaries

ถ้ามี architecture เดิม ให้ preserve เป็นหลักจนกว่าจะมี approved migration plan

---

## 7. PostgreSQL Guidelines

- use official/pinned supported version
- runtime app user ห้ามเป็น superuser/owner โดยไม่จำเป็น
- migration/owner role แยกจาก runtime เมื่อเหมาะสม
- parameterized queries เท่านั้น
- use FK/UNIQUE/CHECK/NOT NULL เพื่อ enforce invariant ที่ DB รู้ได้
- RLS ใช้เมื่อ threat model/multi-tenant architecture ต้องการจริง
- encoding UTF8
- instant timestamps prefer `timestamptz`
- inspect `EXPLAIN (ANALYZE, BUFFERS)` สำหรับ hot query ใน production-like/test environment อย่างระมัดระวัง
- monitor vacuum/analyze/bloat/locks/connections

---

## 8. MySQL Guidelines

- runtime app user ห้ามใช้ root
- use `utf8mb4`
- เลือก collation ตาม language/search semantics
- parameterized query/ORM
- understand InnoDB transaction/locking/isolation semantics
- inspect `EXPLAIN ANALYZE`/optimizer plan ตาม version
- อย่านำ PostgreSQL-specific RLS/extensions/syntax มาใช้

---

## 9. SQLite Guidelines

เหมาะกับ:

- local app
- embedded
- tests บางประเภท
- small single-node workload

ระวัง:

- concurrency/write model แตกต่างจาก PostgreSQL/MySQL
- SQLite test ไม่ใช่หลักฐานแทน production DB locking/isolation semantics
- migration/type behavior อาจต่าง

critical integration tests ต้องใช้ engine เดียวกับ production เมื่อ semantics สำคัญ

---

## 10. MongoDB Guidelines

เลือกเมื่อ document model/query pattern ได้ประโยชน์จริง เช่น aggregate document เปลี่ยน schema บ่อยและ transaction join ไม่ใช่แกนหลัก

กฎ:

- schema validation ยังควรมี
- design document boundary จาก query/update pattern
- ห้าม embed ทุกอย่างจน document โตไม่มีขอบเขต
- indexes ตาม query จริง
- multi-document transactions ใช้ได้ แต่ถ้าระบบพึ่ง relational joins/constraints/transactions หนัก PostgreSQL อาจเหมาะกว่า
- `_id`/public ID exposure ตาม threat model
- backup/restore/PITR ตาม deployment

---

## 11. Redis Guidelines

Redis ไม่ใช่ relational database replacement โดย default

แยก failure semantics ตามบทบาท:

### Cache-only

- cache-aside ใช้ได้
- fallback DB อาจเหมาะ
- stale/TTL/invalidation ต้องออกแบบ

### Security / Rate Limit State

- ห้าม fail-open อัตโนมัติเมื่อ Redis ล่ม
- failure behavior ต้อง explicit ตาม threat model

### Distributed Coordination / Lock

- ห้าม assume lock success เมื่อ backend unavailable
- ต้องเข้าใจ lease/expiry/fencing/idempotency semantics

### Queue / Stream

- delivery semantics, retry, dead-letter, idempotency ต้องชัด

---

## 12. Search / Analytics / Vector Datastores

### OpenSearch / Elasticsearch

- ใช้เป็น search index ไม่ใช่ source of truth ของ transaction โดย default
- sync ผ่าน outbox/event/reindex strategy
- index mapping/analyzer/versioning ต้อง managed

### ClickHouse / Warehouse

- ใช้สำหรับ analytical scans/aggregations
- อย่าเอา OLTP booking/payment mutation ไปอยู่ columnar analytics DB โดยไม่มีเหตุผล
- define ingestion latency และ reconciliation

### Vector Search

- ถ้า data source อยู่ PostgreSQL และ scale ยังไม่มาก ให้พิจารณา pgvector ก่อน
- specialized vector DB ใช้เมื่อ scale/query/latency/feature ต้องการจริง
- original document/source of truth แยกจาก embedding index
- re-embedding/version/model metadata ต้อง traceable

---

## 13. Object Storage

media/blob/large file ไม่ควรเก็บใน application table เป็น Base64 โดย default

ใช้:

- S3-compatible storage
- Cloudflare R2
- cloud object storage
- dedicated media service

DB เก็บ stable key/metadata/checksum/ownership/lifecycle state

private data ใช้ ACL/signed URL ตาม threat model

---

## 14. Schema Design Principles

- model invariant ก่อน table shape
- normalize เป็น baseline; denormalize เมื่อมี evidence/read workload ชัด
- primary key เลือก UUID/ULID/bigint ตาม distributed/public/ordering requirement
- internal ID ไม่จำเป็นต้องเป็น public ID
- FK ใช้เมื่อ ownership/consistency อยู่ DB เดียวกันและ semantics เหมาะ
- CHECK constraints สำหรับ state/range invariant ที่ DB enforce ได้
- nullable ต้องมี business meaning
- money ใช้ integer minor unit หรือ fixed decimal ตาม currency/domain; ห้าม binary float
- counters/versions ใช้ type ที่ไม่ overflow ง่าย

### Timestamps

ไม่บังคับทุก table ต้องมี `created_at`/`updated_at`

ใช้เมื่อ semantic/audit/query มีประโยชน์

- mutable entity มักมี created/updated
- immutable ledger/event อาจมีเพียง occurred/created timestamp
- join table อาจไม่ต้อง updated_at

---

## 15. Time / Timezone Rules

แยก concept:

1. **Instant in time** → UTC / `timestamptz`
2. **Business local date/time** → local date/time + IANA timezone เมื่อ semantics ต้องรักษา
3. **Recurring local schedule** → wall-clock + timezone/rule

ห้าม assume ว่า “เก็บ UTC อย่างเดียว” แก้ทุก business timezone problem

ตัวอย่าง:

- payment created_at → instant UTC
- flight departure 09:00 Tokyo → schedule + `Asia/Tokyo`
- hotel check-in date → business/local date semantics

DST/boundary test ต้องมีเมื่อ region/timezone เกี่ยวข้อง

---

## 16. Transaction & Concurrency Design

สำหรับ booking/inventory/payment/financial flows ต้องออกแบบ invariant ก่อน code

พิจารณา:

- ACID transaction boundary
- isolation level
- optimistic locking/version
- `SELECT ... FOR UPDATE`
- `SKIP LOCKED` สำหรับ worker queue เมื่อเหมาะ
- unique constraint เป็น final race guard
- idempotency key
- deadlock retry
- serialization failure retry
- lease/visibility timeout

### Golden Rule

**ตรวจ availability แล้ว update แยก transaction เป็น race condition ได้**

อย่าพึ่ง application `if` อย่างเดียว ถ้า DB constraint/lock enforce ได้

---

## 17. Isolation Level

Agent ต้องเข้าใจ engine จริง:

- Read Committed
- Repeatable Read
- Serializable

อย่าเลือก Serializable ทุก transaction เพื่อ “ปลอดภัยที่สุด” โดยไม่วัด contention/retry

critical transaction ต้องมี concurrency integration test บน production-equivalent DB

---

## 18. Idempotency

สำคัญกับ:

- payment
- booking creation
- webhook
- external API mutation
- queue consumer

ออกแบบ:

```text
idempotency key
+ actor/operation scope
+ request fingerprint เมื่อเหมาะ
+ stored result/status
+ expiry/lifecycle
```

ห้าม auto-retry non-idempotent mutation โดยไม่มีกลไก

---

## 19. Outbox / Cross-System Consistency

เมื่อ transaction DB ต้องส่ง event ไป Kafka/search/analytics/อีก DB:

**หลีกเลี่ยง dual write:**

```text
DB commit
then publish
```

ที่ไม่มี recovery

พิจารณา Transactional Outbox:

```text
business mutation + outbox row
→ same DB transaction
→ publisher/worker
→ broker/search/secondary DB
→ idempotent consumer
```

ต้องมี retry/dead-letter/reconciliation ตาม criticality

---

## 20. Indexing Strategy

index ตาม query จริง ไม่ใช่ทุก column

พิจารณา:

- B-tree baseline
- composite index order
- partial index
- covering/include index
- GIN/GiST/BRIN ตาม data/query
- unique index เพื่อ invariant

ตรวจ:

- selectivity/cardinality
- scan type
- rows estimated vs actual
- sort/hash memory
- index write amplification

ห้าม optimize จาก intuition อย่างเดียว

---

## 21. Query Plan & Performance

สำหรับ hot query:

```text
query latency
p50/p95/p99
rows scanned
buffer hit/read
CPU
lock wait
pool wait
DB connections
```

ใช้ `EXPLAIN` / engine profiling tools

ระวัง `EXPLAIN ANALYZE` บน destructive/expensive production query เพราะมัน execute query จริง

N+1 ต้องตรวจที่ ORM/data layer ไม่ใช่แค่ database

---

## 22. Connection Pooling

pool size ต้อง tune ร่วมกับ DB capacity ไม่ใช่ request concurrency ตรง ๆ

คิดจาก:

- DB max connections
- number of app instances
- transaction/query latency
- worker/background connections
- admin/maintenance reserve

ใช้ PgBouncer/ProxySQL/managed pooler เมื่อ architecture ได้ประโยชน์

serverless ที่ instance burst สูงต้องระวัง connection storm

---

## 23. Migration Safety

### Rules

- migration immutable หลัง shared/applied; สร้าง migration ใหม่
- migration reproducible จาก empty DB
- migration checksum/history ต้องรักษา
- schema change ต้อง compatible กับ running app ตาม rollout model
- destructive migration ต้อง backup/approval/rollback strategy

### Zero-Downtime Pattern

```text
EXPAND
→ เพิ่ม nullable/new structure ที่ backward-compatible

DEPLOY/BACKFILL
→ code รองรับทั้งเก่าและใหม่ + backfill แบบ bounded

SWITCH
→ เปลี่ยน read/write path

CONTRACT
→ ลบ legacy หลัง verify
```

ระวัง:

- table rewrite
- long exclusive lock
- large backfill transaction
- index creation locking
- NOT NULL เพิ่มทันทีบน table ใหญ่

ใช้ online/concurrent mechanism ตาม engine/version เมื่อเหมาะ

---

## 24. Seed / Fixtures

- seed demo/reference data ต้องแยกจาก migration เมื่อ semantics ต่างกัน
- seed ต้อง idempotent หรือ explicit guarded reset ตาม environment
- normal production startup ไม่ควร seed demo data อัตโนมัติ
- integration test fixture ต้อง unique/scoped และ cleanup เฉพาะ ownership ของตัวเอง
- ห้าม global DELETE/TRUNCATE บน shared DEV

---

## 25. Environment Separation

แยกอย่างน้อย:

```text
DEV
TEST
STAGING (ถ้ามี)
PRODUCTION
```

กฎ:

- test DB ต้องไม่ fallback ไป DEV
- destructive test มี guard ตรวจ host/port/database/environment
- production credentials ห้ามอยู่ local test config/CI log
- localhost ไม่ได้แปลว่า disposable
- migration role กับ runtime role แยกเมื่อ project รองรับ

---

## 26. Least-Privilege Database Roles

แนะนำ conceptual roles:

```text
bootstrap/owner
→ infrastructure only

migrator
→ schema/migration ownership

runtime
→ SELECT/INSERT/UPDATE/DELETE เท่าที่ app ต้องใช้

read-only/reporting
→ scoped read
```

runtime ไม่ควรมี:

- SUPERUSER
- CREATEDB
- CREATEROLE
- schema ownership
- arbitrary DDL
- migration-ledger mutation

ใช้ column-level grants เมื่อข้อมูล/security critical และคุ้ม complexity

---

## 27. Secrets / TLS

- `.env` ไม่ commit
- `.env.example` ไม่มี real secret
- production ใช้ secret manager/platform env
- DB connection ผ่าน TLS เมื่อ network boundary ต้องการ
- cert verification ห้ามปิดเพียงเพื่อแก้ connection ง่าย ๆ
- connection string/password/token ห้าม log
- credential rotation plan ตาม environment criticality

---

## 28. Docker Local Development

Docker Compose เป็นตัวเลือกที่ดีเพื่อ reproducibility แต่ไม่ใช่ข้อบังคับทุก project

ถ้าใช้:

- pin supported image version
- DB host publication local ควร bind loopback เช่น `127.0.0.1`
- app container ใช้ service hostname/internal port
- healthcheck
- named volume สำหรับ DEV persistent data
- TEST สามารถใช้ tmpfs/disposable volume เมื่อเหมาะ
- ห้าม hardcode secret

ตัวอย่าง PostgreSQL:

```yaml
services:
  db:
    image: postgres:<supported-version>
    ports:
      - "127.0.0.1:${POSTGRES_HOST_PORT:-5433}:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
```

ห้ามใช้ `docker compose down -v` กับ valuable DEV volume โดยไม่ตั้งใจ/อนุมัติ

---

## 29. Database Must Not Be Public by Default

Production database:

```text
Internet
   X
Database port
```

application access:

```text
Web Edge / Load Balancer
        ↓
Application services
        ↓ private network / TLS
Database
```

admin access ใช้ private network/VPN/SSH tunnel/bastion/provider console ตาม architecture

รายละเอียด reverse proxy/server topology ให้อยู่ใน `SERVER.md`

---

## 30. Backup / Restore / PITR

Backup ที่ restore ไม่ได้ = ไม่มี backup

กำหนด:

- retention
- encryption
- off-host/off-site copy
- backup frequency
- PITR/WAL/binlog ตาม engine
- restore procedure
- restore drill

### RPO / RTO

- **RPO**: ยอมเสียข้อมูลย้อนหลังได้เท่าไร
- **RTO**: ยอม downtime ได้นานเท่าไร

ระดับ production/enterprise ต้องระบุ target ไม่ใช่แค่ “มี backup”

---

## 31. Replication / HA / Failover

เพิ่มเมื่อ SLA/traffic/availability ต้องการจริง

พิจารณา:

- primary/replica
- synchronous/asynchronous replication
- read replica lag
- automated failover
- split-brain prevention
- connection routing
- failback/recovery drill

ห้ามส่ง read-after-write query ไป replica ที่ lag หาก workflow ต้อง strong consistency

---

## 32. Partitioning / Sharding

### Partitioning

ใช้เมื่อ table size/query/retention pattern มี evidence เช่น time-range pruning

### Sharding

ใช้เมื่อ single primary architecture ไม่ตอบ scale จริงหลัง optimize แล้ว

ต้องมี:

- shard key
- routing
- rebalance
- cross-shard query/transaction semantics
- operational tooling

**ห้าม shard ตั้งแต่เริ่มเพราะคาดว่า “อนาคตระบบใหญ่”**

---

## 33. Data Lifecycle / Privacy

กำหนด:

- retention period
- deletion/anonymization
- legal/audit retention
- PII classification
- encryption at rest/in transit
- backup deletion implications
- tenant isolation
- access auditing

soft delete ไม่ใช่คำตอบทุกกรณี และไม่ได้เท่ากับ privacy deletion

---

## 34. Observability

monitor อย่างน้อยตาม criticality:

- connections / pool utilization
- query latency
- slow queries
- lock waits / deadlocks
- transaction duration
- CPU / memory
- disk / IOPS / space growth
- cache hit ratio
- replication lag
- vacuum/analyze/bloat (PostgreSQL)
- error/timeout rate
- backup success

alert ต้อง actionable ไม่ใช่เก็บ metric อย่างเดียว

---

## 35. Test Strategy

### Unit

business logic ไม่ต้อง DB จริง

### Repository / Integration

ใช้ production-equivalent engine สำหรับ:

- constraints
- transaction
- locking
- isolation
- SQL behavior
- migrations

### Concurrency Tests

booking/payment/inventory/lease ต้องมี deterministic concurrency test

- barrier/lock coordination
- ห้าม arbitrary sleep เป็นหลักฐาน
- test both linearization orderings เมื่อ invariant ต้องการ

### Migration Tests

- empty DB migration
- upgrade from realistic prior version เมื่อ critical
- permission/ownership tests

---

## 36. ORM / Driver Selection

ใช้ของ project เป็นหลัก

ตัวอย่าง:

- Rust: SQLx / Diesel
- Java: JDBC/JdbcClient/jOOQ/JPA-Hibernate
- C#: Npgsql / EF Core / Dapper
- TypeScript: Prisma / Drizzle / Kysely / node-postgres
- Python: SQLAlchemy / psycopg
- Go: pgx / database/sql / sqlc

กฎ:

- ORM productivity ไม่แทน SQL understanding
- critical query ต้อง inspect generated SQL
- N+1 / lazy loading / implicit transaction ต้อง review
- raw SQL ใช้ parameterized statement

---

## 37. API / Data Access Security

- backend authorization ก่อน query sensitive resource
- multi-tenant query ต้อง enforce tenant boundary server-side/DB policy ตาม architecture
- parameterized query
- generic client error; raw DB errorsไม่ออก client
- log ห้ามมี password/token/connection string/PII เกินจำเป็น
- RLS ใช้เมื่อเหมาะและต้อง test policy จริง
- service/admin credential ไม่ส่ง frontend

---

## 38. CI Database Gates

ตาม project criticality CI ควรทำ:

```text
start ephemeral TEST database
→ provision roles
→ run migrations
→ verify schema/ownership/ACL
→ run repository/integration tests
→ run concurrency/security tests
→ teardown disposable TEST environment
```

ห้ามใช้ DEV/production DB ใน CI

migration count/schema count ใช้เฉพาะเมื่อ project มี reason; อย่าฮาร์ดโค้ด generic count ใน guide

---

## 39. Production Readiness Checklist

- [ ] datastore แต่ละตัวมี responsibility/source of truth ชัด
- [ ] multi-database sync/consistency strategy ชัดเจน
- [ ] transaction/concurrency invariants ถูก enforce และ test
- [ ] constraints/indexes ตรง query/business rules
- [ ] hot query มี query-plan evidence
- [ ] connection pool มี sizing rationale
- [ ] migrations backward-compatible/rollout-safe ตาม criticality
- [ ] test DB แยกจาก DEV/PROD และมี guard
- [ ] runtime DB role least privilege
- [ ] DB ไม่ expose public โดยไม่จำเป็น
- [ ] secrets/TLS ถูกต้อง
- [ ] backup + restore drill ผ่าน
- [ ] RPO/RTO ถูกกำหนดเมื่อ production critical
- [ ] PITR/replication/failover ตาม SLA
- [ ] monitoring/alerts พร้อม
- [ ] data retention/privacy lifecycle ชัด
- [ ] CI migration/integration/security gates ผ่าน
- [ ] ไม่มี dump/secret/debug artifacts ใน Git

---

## 40. Agent Completion Report

ตอบเป็นภาษาไทยแบบกระชับโดยระบุ:

1. datastore(s) ที่เลือกและเหตุผล
2. source of truth และ consistency model
3. schema/migrations/indexes ที่เปลี่ยน
4. transaction/concurrency strategy
5. permissions/security boundary
6. migration/seed commands
7. backup/restore impact
8. query/performance evidence
9. test/integration/concurrency results
10. DEV/TEST/PROD safety
11. files changed
12. blockers/deviations และสิ่งที่ยังไม่ได้ verify จริง

ห้ามแสดง secret จริง
