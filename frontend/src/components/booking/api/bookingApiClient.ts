const API_BASE_URL =
  process.env.NEXT_PUBLIC_X_FLY_API_URL ?? "http://localhost:8080/api/v1";

type ApiFieldError = {
  code: string;
  field: string;
  passenger: number;
};

type ApiErrorResponse = {
  error?: {
    code?: string;
    conflictingSeats?: string[];
    fieldErrors?: ApiFieldError[];
    message?: string;
  };
};

class BookingApiError extends Error {
  readonly code: string;
  readonly conflictingSeats: string[];
  readonly fieldErrors: ApiFieldError[];
  readonly status: number;

  constructor({
    code,
    conflictingSeats = [],
    fieldErrors = [],
    message,
    status,
  }: {
    code: string;
    conflictingSeats?: string[];
    fieldErrors?: ApiFieldError[];
    message: string;
    status: number;
  }) {
    super(message);
    this.name = "BookingApiError";
    this.code = code;
    this.conflictingSeats = conflictingSeats;
    this.fieldErrors = fieldErrors;
    this.status = status;
  }
}

const requestJson = async <Result>(path: string, init?: RequestInit) => {
  const response = await fetch(`${API_BASE_URL}${path}`, {
    ...init,
    cache: "no-store",
    credentials: "include",
    headers: {
      ...(init?.body ? { "Content-Type": "application/json" } : {}),
      ...init?.headers,
    },
  });

  if (!response.ok) {
    const payload = (await response.json().catch(() => ({}))) as ApiErrorResponse;
    throw new BookingApiError({
      code: payload.error?.code ?? "BOOKING_REQUEST_FAILED",
      conflictingSeats: payload.error?.conflictingSeats,
      fieldErrors: payload.error?.fieldErrors,
      message: payload.error?.message ?? "The booking request failed.",
      status: response.status,
    });
  }

  return (await response.json()) as Result;
};

export { API_BASE_URL, BookingApiError, requestJson };
export type { ApiErrorResponse, ApiFieldError };
