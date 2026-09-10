import { forwardAdminApiClientRequest } from "@/lib/admin/adminBackend";

export const GET = (request: Request) => forwardAdminApiClientRequest(request, "/admin/api-clients");
export const POST = (request: Request) => forwardAdminApiClientRequest(request, "/admin/api-clients");
