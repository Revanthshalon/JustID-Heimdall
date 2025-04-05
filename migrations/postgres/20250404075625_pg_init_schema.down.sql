-- Drop foreign keys first
ALTER TABLE public.heimdall_relation_tuples
  DROP CONSTRAINT IF EXISTS heimdall_relation_tuples_nid_fk;

-- Drop indexes
DROP INDEX IF EXISTS heimdall_relation_tuples_full_idx;
DROP INDEX IF EXISTS heimdall_relation_tuples_reverse_subject_ids_idx;
DROP INDEX IF EXISTS heimdall_relation_tuples_reverse_subject_sets_idx;
DROP INDEX IF EXISTS heimdall_relation_tuples_subject_ids_idx;
DROP INDEX IF EXISTS heimdall_relation_tuples_subject_sets_idx;
DROP INDEX IF EXISTS schema_migration_version_idx;
DROP INDEX IF EXISTS schema_migration_version_self_idx;

-- Drop primary key constraints
ALTER TABLE IF EXISTS public.heimdall_relation_tuples
  DROP CONSTRAINT IF EXISTS heimdall_relation_tuples_pkey;

ALTER TABLE IF EXISTS public.heimdall_uuid_mappings
  DROP CONSTRAINT IF EXISTS heimdall_uuid_mappings_pkey;

-- Drop tables in reverse order of creation (considering dependencies)
DROP TABLE IF EXISTS public.schema_migration;
DROP TABLE IF EXISTS public.heimdall_relation_tuples;
DROP TABLE IF EXISTS public.heimdall_uuid_mappings;
DROP TABLE IF EXISTS public.networks;
