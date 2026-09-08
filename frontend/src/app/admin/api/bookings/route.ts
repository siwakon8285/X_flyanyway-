import { forwardAdminBookingRequest } from "@/lib/admin/adminBackend";

export const GET = (request: Request) => forwardAdminBookingRequest(request, "/admin/bookings");
