-- Add migration script here
CREATE TABLE IF NOT EXISTS "public"."asset_type" (
    "id" serial PRIMARY KEY,
    "name" text UNIQUE NOT NULL,
    "description" text NOT NULL DEFAULT '',
    "is_active" boolean NOT NULL DEFAULT FALSE,
    "created_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);

COMMENT ON COLUMN "public"."asset_type"."id" IS '主键自增id';

COMMENT ON COLUMN "public"."asset_type"."name" IS '资产类型名称';

COMMENT ON COLUMN "public"."asset_type"."description" IS '资产类型说明';

COMMENT ON COLUMN "public"."asset_type"."is_active" IS '是否启用';

COMMENT ON COLUMN "public"."asset_type"."created_at" IS '创建时间';

COMMENT ON COLUMN "public"."asset_type"."updated_at" IS '更新时间';

COMMENT ON TABLE "public"."asset_type" IS '资产类型表';

INSERT INTO "public"."asset_type" ("name", "description", "is_active")
    VALUES ('GOLD', '金币', 't'),
    ('SILVER', '银币', 't'),
    ('COPPER', '铜币', 't');
