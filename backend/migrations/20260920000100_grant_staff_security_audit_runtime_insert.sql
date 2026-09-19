GRANT INSERT (
    action,
    actor_staff_user_id,
    session_id,
    permission_code,
    request_id
)
ON TABLE public.staff_security_audit
TO x_fly_runtime;
