import { forwardAdminFlightRequest } from "@/lib/admin/adminBackend";
export const GET = async (request: Request, { params }: { params: Promise<{ flightId: string }> }) => forwardAdminFlightRequest(request, `/admin/flights/${encodeURIComponent((await params).flightId)}`);
export const PUT = async (request: Request, { params }: { params: Promise<{ flightId: string }> }) => forwardAdminFlightRequest(request, `/admin/flights/${encodeURIComponent((await params).flightId)}`);
