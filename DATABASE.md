# DATABASE.md

> ส่งไฟล์นี้ให้ AI Agent เช่น Codex หรือ Claude อ่านก่อนเริ่มงาน Database / Data Layer
> Agent ต้องตรวจสอบเทคโนโลยีและโครงสร้างจริงของโปรเจกต์ก่อนแก้ไขเสมอ

---

## 1. บทบาทของ Agent

คุณคือ Senior Backend & Database Engineer มีหน้าที่ออกแบบ ติดตั้ง เชื่อมต่อ และตรวจสอบฐานข้อมูลให้พร้อมสำหรับการพัฒนาแบบทีม โดยยึดหลักต่อไปนี้:

- Security-first และห้ามเปิดเผย secret
- ใช้ Docker Compose เพื่อให้ทีมใช้ environment ใกล้เคียงกัน
- ใช้ migration แชร์ schema และ seed แชร์ข้อมูลเริ่มต้น
- ตรวจ stack เดิมก่อนเลือกเครื่องมือ ห้ามเปลี่ยน framework, ORM หรือฐานข้อมูลโดยพลการ
- วางแผนก่อนแก้ไข และรักษาโค้ดเดิมที่ไม่เกี่ยวข้อง
- หลังแก้ไขต้องรันตรวจสอบจริง ห้ามสรุปว่าสำเร็จจากการอ่านไฟล์เท่านั้น

---

## 2. ตรวจสอบโปรเจกต์ก่อนลงมือ

ตรวจสอบอย่างน้อย:

1. ภาษาและ framework ของ application
2. ORM หรือ driver เช่น Prisma, Drizzle, SQLx, Diesel, TypeORM, Sequelize, Django ORM หรือ Eloquent
3. มี `Dockerfile`, `compose.yaml`, `.env.example`, migration หรือ seed อยู่แล้วหรือไม่
4. โปรเจกต์กำหนด PostgreSQL, MySQL หรือ Supabase ไว้แล้วหรือไม่
5. port ที่ใช้งานอยู่
6. คำสั่ง build, test, migration และ seed จาก config/README

ถ้ามีเทคโนโลยีเดิม ให้ใช้ของเดิมเป็นหลักและห้ามสร้างระบบซ้ำ หากยังไม่ระบุฐานข้อมูลและการเลือกกระทบ architecture ให้เสนอทางเลือกและถามผู้ใช้ก่อนลงมือ

---

## 3. เลือกฐานข้อมูลเพียงแนวทางเดียว

### A — Supabase

เหมาะกับ MVP หรือโปรเจกต์ที่ต้องการ Auth, Storage และ PostgreSQL แบบ managed

- ใช้ Supabase Auth ตามระบบเดิม
- เปิด RLS สำหรับตารางที่ client เข้าถึงผ่าน Supabase API
- เขียน policy ตามสิทธิ์ user/tenant ห้ามเปิดกว้างโดยไม่จำเป็น
- `service_role` ใช้เฉพาะ backend และห้ามส่งไป frontend
- เก็บ secret ใน environment/secret manager
- ใช้ Supabase CLI migrations หรือ migration tool เดิม

```sql
ALTER TABLE orders ENABLE ROW LEVEL SECURITY;

CREATE POLICY "users_can_read_own_orders"
ON orders FOR SELECT
USING (auth.uid() = user_id);
```

### B — PostgreSQL ใน Docker

- ใช้ official PostgreSQL image และ pin เวอร์ชัน
- ห้าม application เชื่อมด้วย superuser `postgres`
- สร้าง app user ที่มีสิทธิ์เท่าที่จำเป็น
- ใช้ parameterized queries หรือ ORM เสมอ
- ใช้ RLS เมื่อ threat model หรือ multi-tenant architecture ต้องการ ไม่บังคับผิดบริบท
- รหัสผ่านผู้ใช้ระบบต้อง hash ด้วย Argon2id หรือกลไกที่ framework แนะนำ
- ตั้ง timezone ของ database เป็น `UTC` เสมอ และแปลงเป็น local timezone ที่ชั้น application/frontend เท่านั้น
- ใช้ encoding `UTF8` (default ของ PostgreSQL รองรับภาษาไทยและ emoji อยู่แล้ว) ตรวจสอบด้วย `SHOW server_encoding;`

### C — MySQL ใน Docker

- ใช้ official MySQL image และ pin เวอร์ชัน
- ห้าม application เชื่อมด้วย `root`
- ใช้ `MYSQL_USER` สำหรับ application และจำกัดสิทธิ์
- ใช้ parameterized queries หรือ ORM เสมอ
- ห้ามนำคำสั่ง PostgreSQL เช่น RLS, `pgcrypto` หรือ `pg_isready` มาใช้
- รหัสผ่านผู้ใช้ระบบต้อง hash ด้วย Argon2id หรือกลไกที่ framework แนะนำ
- ตั้ง character set เป็น `utf8mb4` และ collation เป็น `utf8mb4_unicode_ci` (หรือ `utf8mb4_0900_ai_ci` สำหรับ MySQL 8+) เสมอ เพื่อรองรับภาษาไทยและ emoji ป้องกันปัญหาอักขระขาดหาย
- ตั้ง timezone ของ database เป็น `UTC` เสมอ และแปลงเป็น local timezone ที่ชั้น application/frontend เท่านั้น

เลือกเพียงหนึ่งแนวทางต่อหนึ่งโปรเจกต์ ห้ามผสมโดยไม่มีเหตุผลทางสถาปัตยกรรมชัดเจน

---

## 4. Local Development ด้วย Docker Compose

เป้าหมายคือให้สมาชิกทีม clone repository แล้วเริ่มระบบได้ด้วย:

```bash
cp .env.example .env
docker compose up --build
```

ให้ Agent:

- สร้างหรือปรับ `Dockerfile` และ `compose.yaml` ตาม stack จริง
- ให้ app และ database อยู่ Compose network เดียวกัน
- app ใน container ใช้ hostname ตามชื่อ service เช่น `db` ไม่ใช่ `localhost`
- เพิ่ม named volume ให้ database
- เพิ่ม database healthcheck และให้ app รอจน database healthy
- ใช้ environment variables สำหรับ credentials/connection string
- ห้าม hardcode secret ใน Compose, Dockerfile หรือ source
- ถ้ามี Compose เดิม ให้แก้ไฟล์เดิม ห้ามสร้างไฟล์ซ้ำ

### Port มาตรฐานสำหรับเครื่องนี้

| ระบบ | Local/Homebrew | Docker host → container |
| --- | ---: | ---: |
| PostgreSQL | `5432` | `5433:5432` |
| MySQL | `3306` | `3307:3306` |

ภายใน Docker:

```text
PostgreSQL: db:5432
MySQL:      db:3306
```

`5433` และ `3307` ใช้สำหรับ pgAdmin, Workbench หรือ app ที่รันบน host เท่านั้น

หากเครื่องนี้มีหลายโปรเจกต์รัน Docker Compose พร้อมกันและ `5433`/`3307` ถูกใช้งานแล้ว ให้ Agent ตรวจสอบ port ว่างก่อนด้วย `lsof -i :<port>` หรือ `docker ps` แล้วเลือก host port ถัดไปที่ว่าง (เช่น `5434`, `3308`) ผ่านตัวแปร `POSTGRES_HOST_PORT` / `MYSQL_HOST_PORT` ใน `.env` ห้ามแก้ internal container port (`5432`/`3306`)

---

## 5. ตัวอย่าง Compose — PostgreSQL

ปรับ image, ชื่อ database/user และคำสั่ง app ตามโปรเจกต์จริง เวอร์ชัน image ด้านล่างเป็นเพียงตัวอย่าง Agent ต้องตรวจสอบเวอร์ชัน stable ล่าสุดที่ใช้งานจริง ณ เวลาที่ทำงาน แทนการ copy ตามเอกสารนี้เสมอ:

```yaml
services:
  db:
    image: postgres:18
    restart: unless-stopped
    environment:
      POSTGRES_DB: ${POSTGRES_DB}
      POSTGRES_USER: ${POSTGRES_USER}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    ports:
      - "${POSTGRES_HOST_PORT:-5433}:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER} -d ${POSTGRES_DB}"]
      interval: 5s
      timeout: 5s
      retries: 10
      start_period: 10s

  app:
    build: .
    env_file:
      - .env
    depends_on:
      db:
        condition: service_healthy

volumes:
  postgres_data:
```

```env
DATABASE_URL=postgresql://app_user:change_me@db:5432/app_db
```

`depends_on: condition: service_healthy` ช่วยให้ container เริ่มตามลำดับ แต่ไม่รับประกันว่า database พร้อมรับ connection 100% ของทุก framework ควรเพิ่ม retry/backoff logic ที่ชั้น application (เช่น retry เชื่อมต่อ 5-10 ครั้ง ห่างกัน 2-3 วินาที) โดยเฉพาะ ORM ที่ไม่มี built-in retry

---

## 6. ตัวอย่าง Compose — MySQL

```yaml
services:
  db:
    image: mysql:9
    restart: unless-stopped
    environment:
      MYSQL_ROOT_PASSWORD: ${MYSQL_ROOT_PASSWORD}
      MYSQL_DATABASE: ${MYSQL_DATABASE}
      MYSQL_USER: ${MYSQL_USER}
      MYSQL_PASSWORD: ${MYSQL_PASSWORD}
    ports:
      - "${MYSQL_HOST_PORT:-3307}:3306"
    volumes:
      - mysql_data:/var/lib/mysql
    healthcheck:
      test: ["CMD-SHELL", "mysqladmin ping -h localhost -u root -p$$MYSQL_ROOT_PASSWORD --silent"]
      interval: 5s
      timeout: 5s
      retries: 20
      start_period: 20s

  app:
    build: .
    env_file:
      - .env
    depends_on:
      db:
        condition: service_healthy

volumes:
  mysql_data:
```

```env
DATABASE_URL=mysql://app_user:change_me@db:3306/app_db
```

ตรวจ syntax ของ password/URL encoding ตาม driver และห้ามใช้ค่าตัวอย่างเป็น secret จริง

---

## 7. Environment และ Secret

ต้องมี `.env.example` ซึ่งใช้ค่าตัวอย่างที่ไม่ใช่ secret และต้องตรวจว่า `.env` ถูก ignore ใน Git

PostgreSQL:

```env
POSTGRES_DB=app_db
POSTGRES_USER=app_user
POSTGRES_PASSWORD=change_me
POSTGRES_HOST_PORT=5433
DATABASE_URL=postgresql://app_user:change_me@db:5432/app_db
```

MySQL:

```env
MYSQL_ROOT_PASSWORD=change_root_password
MYSQL_DATABASE=app_db
MYSQL_USER=app_user
MYSQL_PASSWORD=change_me
MYSQL_HOST_PORT=3307
DATABASE_URL=mysql://app_user:change_me@db:3306/app_db
```

กฎบังคับ:

- Commit: `.env.example`, Compose, Dockerfile, migrations, seed และ README
- ห้าม Commit: `.env`, credential, private key, token หรือ database dump ที่มีข้อมูลจริง
- ถ้าพบ secret ในไฟล์ที่ track/Git history ให้หยุดและแจ้งผู้ใช้โดยไม่แสดง secret ซ้ำ
- production ต้องใช้ secret manager หรือ environment ของ deployment platform

---

## 8. Migration และ Seed

- ใช้ migration system ของ ORM/framework เดิม
- Rust + SQLx ใช้ `sqlx migrate`; Supabase ใช้ Supabase CLI
- Prisma, Drizzle, Django, Laravel หรือเครื่องมืออื่น ใช้คำสั่ง native
- ห้ามแก้ migration ที่ merge หรือใช้ร่วมกันแล้ว ให้สร้าง migration ใหม่
- migration ต้อง reproducible และทำงานกับ database ว่าง
- rollback/down ให้ทำตามแนวทางของเครื่องมือ และเตือนก่อนการลบข้อมูล
- seed ควร idempotent และต้องไม่มีข้อมูลจริงหรือ secret
- schema แชร์ผ่าน migration; ตัวอย่างข้อมูลแชร์ผ่าน seed; runtime data/volume ไม่แชร์ผ่าน Git

---

## 9. Naming Convention

เพื่อให้ schema อ่านง่ายและสอดคล้องกันทั้งทีม ใช้แนวทางนี้เมื่อโปรเจกต์ยังไม่มี convention เดิม:

- ชื่อตารางและคอลัมน์ใช้ `snake_case` เช่น `order_items`, `created_at`
- ชื่อตารางเป็นพหูพจน์ เช่น `users`, `orders`
- Primary key ชื่อ `id`, Foreign key ชื่อ `<table_singular>_id` เช่น `user_id`
- ทุกตารางควรมี `created_at` และ `updated_at` (timestamp, UTC) เป็นมาตรฐาน
- ถ้าใช้ soft delete ให้ใช้คอลัมน์ `deleted_at` (nullable timestamp) แทนการลบจริง และ index คอลัมน์นี้ถ้า query บ่อย
- ชื่อ index/constraint ให้สื่อความหมาย เช่น `idx_orders_user_id`, `fk_orders_user_id`

หากโปรเจกต์เดิมมี convention อยู่แล้ว ให้ยึดตามของเดิม ห้ามเปลี่ยนกลางทาง

---

## 10. Test Database แยกจาก Dev Database

ป้องกันไม่ให้การรัน test ไปทับหรือลบข้อมูล dev โดยไม่ตั้งใจ:

- ใช้ database คนละชื่อสำหรับ test เช่น `app_db_test` (แยกจาก `app_db`) หรือใช้ schema/service แยกใน Compose
- กำหนด `DATABASE_URL` สำหรับ test ผ่าน environment variable แยก เช่น `TEST_DATABASE_URL` ใน `.env.example`
- migration ที่รันกับ dev ต้องรันกับ test database ได้เหมือนกัน (reproducible)
- test suite ควร reset/seed ข้อมูลก่อนแต่ละรอบหรือใช้ transaction rollback เพื่อความ idempotent
- ห้าม hardcode ให้ test ชี้ไปที่ database เดียวกับ dev/production โดยเด็ดขาด

---

## 11. Backup และ Restore (Local/Dev)

แม้เป็น environment สำหรับ dev ก็ควรมีแนวทางสำรองข้อมูลเบื้องต้น เผื่อ volume เสียหายหรือ migration ผิดพลาด:

### PostgreSQL

```bash
# Backup
docker compose exec db pg_dump -U ${POSTGRES_USER} -d ${POSTGRES_DB} > backup.sql

# Restore
cat backup.sql | docker compose exec -T db psql -U ${POSTGRES_USER} -d ${POSTGRES_DB}
```

### MySQL

```bash
# Backup
docker compose exec db mysqldump -u ${MYSQL_USER} -p${MYSQL_PASSWORD} ${MYSQL_DATABASE} > backup.sql

# Restore
cat backup.sql | docker compose exec -T db mysql -u ${MYSQL_USER} -p${MYSQL_PASSWORD} ${MYSQL_DATABASE}
```

กฎ:

- ไฟล์ backup (`*.sql`, `*.dump`) ห้าม commit เข้า Git เพราะอาจมีข้อมูลจริงปนอยู่ ให้เพิ่มใน `.gitignore`
- สำหรับ production ต้องใช้ automated backup ของ managed service (เช่น Supabase point-in-time recovery) หรือ scheduled backup job แยกต่างหาก ไม่ใช่คำสั่ง manual ด้านบน
- ทดสอบ restore เป็นระยะเพื่อให้มั่นใจว่า backup ใช้งานได้จริง

---

## 12. Connection Pooling และ TLS

- Dev/local: connection pool ที่ ORM มีให้ในตัว (เช่น Prisma, SQLx pool) เพียงพอ ไม่จำเป็นต้องตั้ง PgBouncer เพิ่ม
- Production หรือ serverless ที่มี concurrent connection สูง: พิจารณาใช้ connection pooler เช่น PgBouncer (PostgreSQL) หรือ ProxySQL (MySQL) เพื่อไม่ให้ database connection limit เต็ม
- Production ต้องเปิดใช้ TLS/SSL สำหรับการเชื่อมต่อ database เสมอ (เช่น `sslmode=require` ใน PostgreSQL connection string, หรือ `ssl-mode=REQUIRED` ใน MySQL) ยกเว้น local Docker ที่อยู่ใน network ปิดของเครื่องเดียวกัน
- Managed service อย่าง Supabase เปิด TLS ให้อัตโนมัติอยู่แล้ว ไม่ต้องตั้งค่าเพิ่ม

---

## 13. เชื่อมต่อ GUI

### pgAdmin → PostgreSQL Docker

```text
Host: localhost หรือ 127.0.0.1
Port: 5433
Database: POSTGRES_DB
Username: POSTGRES_USER
Password: POSTGRES_PASSWORD จาก .env ของผู้ใช้
```

### MySQL Workbench → MySQL Docker

```text
Hostname: 127.0.0.1
Port: 3307
Username: MYSQL_USER
Password: MYSQL_PASSWORD จาก .env ของผู้ใช้
```

อย่าใช้ `5432` หรือ `3306` สำหรับ Docker บนเครื่องนี้ เพราะ local database ใช้อยู่แล้ว

---

## 14. API และ Data Access Security

- ใช้ route versioning ตาม convention เดิม เช่น `/api/v1/`
- validate input ก่อนส่งเข้า database
- ทุก query ต้อง parameterized หรือผ่าน ORM
- ใช้ transaction กับงานที่ต้องสำเร็จ/ล้มเหลวพร้อมกัน
- บังคับ authorization ที่ backend ห้ามเชื่อ user/role จาก frontend โดยตรง
- ให้สิทธิ์ database user ต่ำที่สุดที่ app ต้องใช้
- log error โดยไม่เผย password, token, connection string หรือข้อมูลอ่อนไหว
- เพิ่ม index ตาม query จริงและตรวจ query plan ก่อน optimization ซับซ้อน

ถ้าโปรเจกต์ยังไม่มี response convention:

```json
{ "data": {} }
```

```json
{ "error": "message", "code": "ERROR_CODE" }
```

---

## 15. ตรวจสอบก่อนส่งงาน

Agent ต้องรันคำสั่งตามโปรเจกต์จริงและรายงานผล/error:

1. ตรวจ Compose:

   ```bash
   docker compose config
   ```

2. Build และเริ่ม container:

   ```bash
   docker compose up --build -d
   ```

3. ตรวจ health:

   ```bash
   docker compose ps
   ```

4. ตรวจ log โดยไม่เผย secret:

   ```bash
   docker compose logs --no-color --tail=200
   ```

5. รัน migration ด้วยคำสั่งของ ORM/framework
6. รัน seed (ถ้ามี)
7. ทดสอบว่า app เชื่อม DB และ query ได้อย่างน้อยหนึ่งรายการ
8. รัน test/lint/type-check ที่เกี่ยวข้อง โดยตรวจว่าชี้ไปที่ test database แยกจาก dev
9. ตรวจว่า `.env` ไม่ถูก track และไม่มี secret ใหม่ใน diff
10. ตรวจว่าไฟล์ backup (ถ้ามีจากการทดสอบ) ไม่ถูก track ใน Git
11. สรุปไฟล์ที่แก้ คำสั่ง ผลตรวจ และค่าที่ผู้ใช้ต้องตั้งเอง

หาก Docker daemon ไม่ทำงาน, dependency ขาด, port ถูกใช้ หรือ permission ไม่พอ ให้รายงาน blocker ตามจริง ห้ามอ้างว่าพร้อมใช้งาน

---

## 16. Definition of Done

- [ ] เลือกฐานข้อมูลตรง requirement และ stack เดิม
- [ ] App และ database รันผ่าน Docker Compose
- [ ] Database มี persistent named volume
- [ ] Healthcheck ผ่าน และ app รอ database พร้อม
- [ ] App ใน container ใช้ `db` และ internal port ถูกต้อง
- [ ] PostgreSQL Docker ใช้ host `5433` หรือ MySQL ใช้ `3307` (หรือ port ว่างถัดไปถ้าชนกัน)
- [ ] มี `.env.example` และไม่มี secret จริงใน Git
- [ ] App ไม่ใช้ `postgres`, `root` หรือ superuser
- [ ] Charset/collation และ timezone (UTC) ตั้งค่าถูกต้อง
- [ ] Migration ทำงานกับ database ว่าง
- [ ] Seed ทำงานและไม่มีข้อมูลจริง
- [ ] Test database แยกจาก dev database อย่างชัดเจน
- [ ] มีขั้นตอน backup/restore ที่ทดสอบแล้วอย่างน้อยหนึ่งครั้ง
- [ ] Production มี TLS/SSL และ connection pooling ตามความเหมาะสม
- [ ] Naming convention ของตาราง/คอลัมน์สอดคล้องกันทั้ง schema
- [ ] pgAdmin/Workbench เชื่อมผ่าน host port ที่กำหนดได้
- [ ] Query ปลอดภัยจาก SQL injection
- [ ] Tests/checks ผ่าน หรือรายงานข้อผิดพลาดตามจริง
- [ ] README อธิบายขั้นตอนสำหรับสมาชิกทีมใหม่

---

## 17. รูปแบบรายงานกลับผู้ใช้

ตอบเป็นภาษาไทยแบบกระชับ โดยระบุ:

1. ฐานข้อมูลที่เลือกและเหตุผล
2. ไฟล์ที่สร้างหรือแก้
3. คำสั่งเริ่มระบบ
4. คำสั่ง migration และ seed
5. วิธีเชื่อม pgAdmin/MySQL Workbench
6. ผล build, healthcheck, migration, seed และ tests
7. blocker หรือค่าที่ผู้ใช้ต้องกำหนดเอง

ห้ามแสดงรหัสผ่านหรือ secret จริงในรายงาน
