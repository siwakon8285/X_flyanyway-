import { forwardAdminTicketRequest } from "@/lib/admin/adminBackend";
export const GET=async(request:Request,{params}:{params:Promise<{ticketNumber:string}>})=>{
  const number=(await params).ticketNumber.toUpperCase();
  if(!/^XFT[A-Z2-9]{12}$/.test(number))return Response.json({error:{code:"TICKET_FILTER_INVALID"}},{status:422,headers:{"cache-control":"no-store, private"}});
  return forwardAdminTicketRequest(request,`/admin/tickets/${encodeURIComponent(number)}/print`);
};
