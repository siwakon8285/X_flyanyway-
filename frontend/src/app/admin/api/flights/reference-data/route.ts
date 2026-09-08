import { forwardAdminFlightRequest } from "@/lib/admin/adminBackend";
export const GET = (request: Request) => forwardAdminFlightRequest(request, "/admin/flights/reference-data");
