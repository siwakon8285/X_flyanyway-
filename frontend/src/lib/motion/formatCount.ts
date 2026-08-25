type CountFormatOptions = {
  decimals?: number;
  prefix?: string;
  suffix?: string;
};

const formatCount = (
  value: number,
  { decimals = 0, prefix = "", suffix = "" }: CountFormatOptions = {},
) => `${prefix}${value.toFixed(decimals)}${suffix}`;

export { formatCount };
export type { CountFormatOptions };
