import { forwardAdminAuthRequest } from "@/lib/admin/adminBackend";

const GET = (request: Request) => forwardAdminAuthRequest(request, "/admin/auth/session");

export { GET };
