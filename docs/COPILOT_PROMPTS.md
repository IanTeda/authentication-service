# Copilot Prompt Templates for authentication_service

This document contains example prompts and templates you can paste into GitHub Copilot or other AI assistants to get predictable, high-quality output for this repository.

Guidelines before you ask:

- Always include the target file path and function signature you want modified.
- Mention whether the change should be accompanied by unit tests or integration tests.
- Include short snippets of relevant surrounding code or the DB schema when possible.

Example 1: Implement an insert_batch function for email verifications

Prompt:

```
File: `src/database/email_verification/insert.rs`
Implement `insert_batch(tx: &mut PgConnection, verifications: Vec<NewEmailVerification>) -> Result<Vec<EmailVerification>, AuthenticationError>`.
- Validate each `NewEmailVerification` for token length and expiry in the future.
- Ensure the function runs inside a transaction and rolls back on error.
- Detect duplicate tokens within the batch before inserting and return `AuthenticationError::ValidationError{field: "token", message: "duplicate tokens in batch"}`.
- Map `sqlx::Error` to `AuthenticationError::DatabaseError { operation: "insert", source }`.
- Add tracing spans that include `batch_size` and `inserted` counts.
- Provide unit tests that cover success, validation error, and DB constraint violation.
```

Example 2: Update model to include `updated_at` and `is_used` field naming

Prompt:

```
File: `src/database/email_verification/model.rs`
Update the `EmailVerification` struct to ensure fields match DB columns: `id`, `user_id`, `token`, `expires_at`, `is_used` (bool), `created_at`, `updated_at`.
- Add `impl EmailVerification { pub fn is_expired(&self) -> bool { ... } }`.
- Add `pub fn is_valid(&self, secret: &Secret<String>, issuer: &str) -> Result<bool, AuthenticationError>` which verifies the token signature and that token `exp` is after `Utc::now()`.
- Add unit tests for `is_expired` and `is_valid` (both valid and invalid tokens).
```

Example 3: Write cursor pagination tests

Prompt:

```
File: `tests/read/email_verifications.rs`
Create tests for `index_cursor` covering:
- Empty table returns empty vector
- Page size limits are respected
- Cursor boundaries (created_at + id) return correct subsequent pages
- Ordering is deterministic (created_at desc, id desc)
```

Quick tips:
- For DB-related code, include the `migrations/` SQL file snippets to ensure AI knows column names and constraints.
- Ask for both unit tests and integration tests when behavior touches the DB.

If you need a new prompt template added here, make a small PR with the new example and a short explanation.
