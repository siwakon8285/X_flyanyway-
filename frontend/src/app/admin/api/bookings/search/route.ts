import { forwardAdminBookingRequest } from "@/lib/admin/adminBackend";

export const POST = (request: Request) => forwardAdminBookingRequest(request, "/admin/bookings/search");
