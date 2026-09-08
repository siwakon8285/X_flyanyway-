import { forwardAdminFlightRequest } from "@/lib/admin/adminBackend";
export const POST = async (request: Request, { params }: { params: Promise<{ flightId: string }> }) => forwardAdminFlightRequest(request, `/admin/flights/${encodeURIComponent((await params).flightId)}/cancel`);
