import { forwardAdminBookingRequest } from "@/lib/admin/adminBackend";

export const GET = async (request: Request, { params }: { params: Promise<{ bookingReference: string }> }) => {
  const reference = (await params).bookingReference.toUpperCase();
  if (!/^XF[A-Z2-9]{8}$/.test(reference)) return Response.json({ error: { code: "BOOKING_FILTER_INVALID" } }, { status: 422 });
  return forwardAdminBookingRequest(request, `/admin/bookings/${encodeURIComponent(reference)}`);
};
