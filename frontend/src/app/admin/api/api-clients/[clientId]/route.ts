import { forwardAdminApiClientRequest } from "@/lib/admin/adminBackend";

const path = async (params: Promise<{ clientId: string }>) =>
  `/admin/api-clients/${encodeURIComponent((await params).clientId)}`;

export const GET = async (request: Request, { params }: { params: Promise<{ clientId: string }> }) =>
  forwardAdminApiClientRequest(request, await path(params));
export const PUT = async (request: Request, { params }: { params: Promise<{ clientId: string }> }) =>
  forwardAdminApiClientRequest(request, await path(params));
