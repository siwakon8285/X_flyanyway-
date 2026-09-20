import { forwardAdminTicketRequest } from "@/lib/admin/adminBackend";

const ticketPattern = /^XFT[A-Z2-9]{12}$/;

export const GET = async (
  request: Request,
  { params }: { params: Promise<{ ticketNumber: string; passengerOrdinal: string }> },
) => {
  const { ticketNumber, passengerOrdinal } = await params;
  const number = ticketNumber.toUpperCase();
  if (!ticketPattern.test(number) || !/^[1-9][0-9]{0,2}$/.test(passengerOrdinal)) {
    return Response.json(
      { error: { code: "BOARDING_PASS_REQUEST_INVALID" } },
      { status: 422, headers: { "cache-control": "no-store, private" } },
    );
  }
  return forwardAdminTicketRequest(
    request,
    `/admin/tickets/${encodeURIComponent(number)}/passengers/${passengerOrdinal}/boarding-pass`,
  );
};

export const POST = async (
  request: Request,
  { params }: { params: Promise<{ ticketNumber: string; passengerOrdinal: string }> },
) => {
  const { ticketNumber, passengerOrdinal } = await params;
  const number = ticketNumber.toUpperCase();
  if (!ticketPattern.test(number) || !/^[1-9][0-9]{0,2}$/.test(passengerOrdinal)) {
    return Response.json(
      { error: { code: "BOARDING_PASS_REQUEST_INVALID" } },
      { status: 422, headers: { "cache-control": "no-store, private" } },
    );
  }
  return forwardAdminTicketRequest(
    request,
    `/admin/tickets/${encodeURIComponent(number)}/passengers/${passengerOrdinal}/boarding-pass`,
  );
};
