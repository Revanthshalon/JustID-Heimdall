/*
 * TABLE: heimdall_relation_tuples
 *
 * PURPOSE:
 *   Core table implementing Google's Zanzibar authorization system, storing relationship tuples
 *   that define who has what permissions on which resources.
 *
 * FEATURES:
 *   - Fine-grained permission control with tuple-based access modeling
 *   - Supports both direct user permissions and group-based (subject set) permissions
 *   - Horizontal scaling via sharding for high-volume authorization systems
 *   - Multi-tenant architecture with complete network isolation
 *   - Optimized for fast permission lookups and traversals
 *
 * USAGE:
 *   - Direct permission check: "Can user X perform relation Y on object Z?"
 *   - Group membership checks: "Is user X a member of group G with relation R?"
 *   - Permission graph traversal for complex authorization decisions
 *   - Used for both read-time permission checks and write-time validation
 */
CREATE TABLE public.heimdall_relation_tuples (
  shard_id UUID NOT NULL, -- Partition key for horizontal scaling
  nid UUID NOT NULL, -- Network ID for multi-tenancy isolation
  namespace VARCHAR(200) NOT NULL, -- Object type (e.g., 'document', 'folder')
  object UUID NOT NULL, -- Specific resource being protected
  relation VARCHAR(64) NOT NULL, -- Permission type (e.g., 'viewer', 'editor')
  subject_id UUID NULL, -- Direct subject (user) ID when applicable
  subject_set_namespace VARCHAR(200) NULL, -- Group type for indirect relationships
  subject_set_object UUID NULL, -- Group ID for indirect relationships
  subject_set_relation VARCHAR(64) NULL, -- Relation within the group
  commit_time TIMESTAMPTZ NOT NULL, -- When this relationship was established
  CONSTRAINT check_heimdall_rt_uuid_subject_type CHECK (
    (
      ((subject_id IS NULL) AND (subject_set_namespace IS NOT NULL) AND (subject_set_object IS NOT NULL) AND (subject_set_relation IS NOT NULL))
      OR
      ((subject_id IS NOT NULL) AND (subject_set_namespace IS NULL) AND (subject_set_object IS NULL) AND (subject_set_relation IS NULL))
    )
);

/*
 * TABLE: heimdall_uuid_mappings
 *
 * PURPOSE:
 *   Provides human-readable translations for the UUIDs used throughout the system.
 *
 * FEATURES:
 *   - Bidirectional mapping between technical UUIDs and readable identifiers
 *   - Enables friendly naming in logs, UI, and API responses
 *
 * USAGE:
 *   - UUID resolution for display purposes
 *   - String-to-UUID lookups for API input processing
 *   - Audit trail enrichment with readable identifiers
 */
CREATE TABLE public.heimdall_uuid_mappings (
  id UUID NOT NULL, -- UUID primary key
  string_representation TEXT NOT NULL -- Human-readable identifier
);

/*
 * TABLE: networks
 *
 * PURPOSE:
 *   Provides multi-tenancy support by creating isolated authorization domains.
 *
 * FEATURES:
 *   - Complete isolation between different tenant environments
 *   - Independent permission models per network
 *   - Lifecycle tracking with timestamps
 *
 * USAGE:
 *   - Tenant onboarding and provisioning
 *   - Permission isolation between organizations
 *   - Resource and security boundary enforcement
 */
CREATE TABLE public.networks (
  id UUID NOT NULL PRIMARY KEY, -- Unique network identifier
  created_at TIMESTAMPTZ NOT NULL, -- Network creation timestamp
  updated_at TIMESTAMPTZ NOT NULL -- Last modification timestamp
);

/*
 * TABLE: schema_migrations
 *
 * PURPOSE:
 *   Manages database schema versioning and migrations.
 *
 * FEATURES:
 *   - Tracks applied schema changes
 *   - Ensures consistent database state across deployments
 *   - Supports safe incremental schema evolution
 *
 * USAGE:
 *   - Migration tracking during deployments
 *   - Version verification during application startup
 *   - Rollback support for failed migrations
 */
CREATE TABLE public.schema_migrations (
  version VARCHAR(48) NOT NULL, -- Migration version identifier
  version_self INTEGER DEFAULT 0 NOT NULL -- Internal versioning counter
);

-- Primary key constraints
ALTER TABLE ONLY public.heimdall_relation_tuples
  ADD CONSTRAINT heimdall_relation_tuples_pkey PRIMARY KEY (shard_id, nid);

ALTER TABLE ONLY public.heimdall_uuid_mappings
  ADD CONSTRAINT heimdall_uuid_mappings_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.networks
  ADD CONSTRAINT networks_pkey PRIMARY KEY (id);

/*
 * INDEX: heimdall_relation_tuples_full_idx
 * PURPOSE: Supports full tuple lookups with all parameters
 * PERFORMANCE: Optimizes exact-match permission checks
 */
CREATE INDEX heimdall_relation_tuples_full_idx ON public.heimdall_relation_tuples USING btree (nid, namespace, object, relation, subject_id, subject_set_namespace, subject_set_object, subject_set_relation, commit_time);

/*
 * INDEX: heimdall_relation_tuples_reverse_subject_ids_idx
 * PURPOSE: Supports reverse permission queries for direct subjects
 * PERFORMANCE: Accelerates "what can user X do?" queries
 */
CREATE INDEX heimdall_relation_tuples_reverse_subject_ids_idx ON public.heimdall_relation_tuples USING btree (nid, subject_id, relation, namespace) WHERE ((subject_set_namespace IS NULL) AND (subject_set_object IS NULL) AND (subject_set_relation IS NULL));

/*
 * INDEX: heimdall_relation_tuples_reverse_subject_sets_idx
 * PURPOSE: Supports reverse permission queries for group members
 * PERFORMANCE: Accelerates "what can members of group G do?" queries
 */
CREATE INDEX heimdall_relation_tuples_reverse_subject_sets_idx ON public.heimdall_relation_tuples USING btree (nid, subject_set_namespace, subject_set_object, subject_set_relation, relation, namespace) WHERE (subject_id IS NULL);

/*
 * INDEX: heimdall_relation_tuples_subject_ids_idx
 * PURPOSE: Accelerates direct subject permission checks
 * PERFORMANCE: Optimizes common "can user X do Y on Z?" permission queries
 */
CREATE INDEX heimdall_relation_tuples_subject_ids_idx ON public.heimdall_relation_tuples USING btree (nid, namespace, object, relation, subject_id) WHERE ((subject_set_namespace IS NULL) AND (subject_set_object IS NULL) AND (subject_set_relation IS NULL));

/*
 * INDEX: heimdall_relation_tuples_subject_sets_idx
 * PURPOSE: Accelerates subject set (group) permission checks
 * PERFORMANCE: Optimizes "can any member of group G do Y on Z?" queries
 */
CREATE INDEX heimdall_relation_tuples_subject_sets_idx ON public.heimdall_relation_tuples USING btree (nid, namespace, object, relation, subject_set_namespace, subject_set_object, subject_set_relation) WHERE (subject_id IS NULL);

/*
 * INDEX: schema_migration_version_idx
 * PURPOSE: Enforces unique migration versions
 * PERFORMANCE: Fast version existence checks
 */
CREATE UNIQUE INDEX schema_migration_version_idx ON public.schema_migration USING btree (version);

/*
 * INDEX: schema_migration_version_self_idx
 * PURPOSE: Supports internal versioning operations
 * PERFORMANCE: Optimizes internal version counter queries
 */
CREATE INDEX schema_migration_version_self_idx ON public.schema_migration USING btree (version_self);

/*
 * CONSTRAINT: heimdall_relation_tuples_nid_fk
 * PURPOSE: Ensures referential integrity between relation tuples and networks
 * BEHAVIOR: Cascading delete when a network is removed
 */
ALTER TABLE ONLY public.heimdall_relation_tuples
  ADD CONSTRAINT heimdall_relation_tuples_nid_fk FOREIGN KEY (nid) REFERENCES public.networks(id) ON UPDATE RESTRICT ON DELETE CASCADE;
