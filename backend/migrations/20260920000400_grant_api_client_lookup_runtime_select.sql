-- Complete the Branch 25 credential lookup allowlist. The repository filters
-- credential rows by api_client_id, so the runtime role needs this read-only
-- column in addition to the credential metadata columns granted previously.
GRANT SELECT (api_client_id)
ON TABLE public.api_client_credentials
TO x_fly_runtime;
