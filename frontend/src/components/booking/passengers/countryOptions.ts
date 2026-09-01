import { callingCodeCountries, countryCallingCode } from "@/components/booking/passengers/countryCallingCodes";

const callingCodeForCountry = (countryCode: string) => countryCallingCode(countryCode);

const callingCodeOptions = callingCodeCountries.map((countryCode) => ({
  countryCode,
  value: countryCallingCode(countryCode)!,
}));

export { callingCodeForCountry, callingCodeOptions };
