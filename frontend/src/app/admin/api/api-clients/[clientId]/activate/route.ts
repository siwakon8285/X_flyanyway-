import { forwardAdminApiClientRequest } from "@/lib/admin/adminBackend";

export const POST = async (request: Request, { params }: { params: Promise<{ clientId: string }> }) =>
  forwardAdminApiClientRequest(request, `/admin/api-clients/${encodeURIComponent((await params).clientId)}/activate`);
