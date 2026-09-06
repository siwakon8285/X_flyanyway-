import { forwardAdminAuthRequest } from "@/lib/admin/adminBackend";

const POST = (request: Request) => forwardAdminAuthRequest(request, "/admin/auth/login");

export { POST };
