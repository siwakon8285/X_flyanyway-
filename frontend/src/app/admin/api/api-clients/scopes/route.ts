import { forwardAdminApiClientRequest } from "@/lib/admin/adminBackend";

export const GET = (request: Request) => forwardAdminApiClientRequest(request, "/admin/api-clients/scopes");
