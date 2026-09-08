import { forwardAdminFlightRequest } from "@/lib/admin/adminBackend";
export const GET = (request: Request) => forwardAdminFlightRequest(request, "/admin/flights");
export const POST = (request: Request) => forwardAdminFlightRequest(request, "/admin/flights");
