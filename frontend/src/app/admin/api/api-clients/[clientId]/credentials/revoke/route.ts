import { revokeApiClientCredential } from "@/lib/admin/adminBackend";

const versionFrom = async (request:Request):Promise<number | null> => {
  const body = await request.json().catch(() => null) as { version?:unknown } | null;
  return typeof body?.version === "number" && Number.isSafeInteger(body.version) && body.version >= 1 ? body.version : null;
};

export const POST = async (request:Request, { params }:{ params:Promise<{ clientId:string }> }) => {
  const version = await versionFrom(request);
  if (version === null) return Response.json({ error:{ code:"API_CLIENT_VALIDATION_FAILED" } }, { status:422, headers:{ "cache-control":"no-store, private" } });
  return revokeApiClientCredential((await params).clientId, version, request);
};
