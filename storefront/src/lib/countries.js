// ISO 3166-1 alpha-2 codes; names come from the browser.
export const COUNTRY_CODES = (
  'AD AE AF AG AI AL AM AO AQ AR AS AT AU AW AX AZ BA BB BD BE BF BG BH BI BJ BL BM BN BO BQ BR BS BT BV BW BY BZ ' +
  'CA CC CD CF CG CH CI CK CL CM CN CO CR CU CV CW CX CY CZ DE DJ DK DM DO DZ EC EE EG EH ER ES ET FI FJ FK FM FO FR ' +
  'GA GB GD GE GF GG GH GI GL GM GN GP GQ GR GS GT GU GW GY HK HM HN HR HT HU ID IE IL IM IN IO IQ IR IS IT JE JM JO JP ' +
  'KE KG KH KI KM KN KP KR KW KY KZ LA LB LC LI LK LR LS LT LU LV LY MA MC MD ME MF MG MH MK ML MM MN MO MP MQ MR MS MT ' +
  'MU MV MW MX MY MZ NA NC NE NF NG NI NL NO NP NR NU NZ OM PA PE PF PG PH PK PL PM PN PR PS PT PW PY QA RE RO RS RU RW ' +
  'SA SB SC SD SE SG SH SI SJ SK SL SM SN SO SR SS ST SV SX SY SZ TC TD TF TG TH TJ TK TL TM TN TO TR TT TV TW TZ UA UG ' +
  'UM US UY UZ VA VC VE VG VI VN VU WF WS XK YE YT ZA ZM ZW'
).split(' ');

function displayNames(locale) {
  try {
    return new Intl.DisplayNames([locale], { type: 'region' });
  } catch (_) {
    return null;
  }
}
const en = displayNames('en');
const de = displayNames('de');

export function countryName(code) {
  try {
    return (code === 'XK' ? 'Kosovo' : en?.of(code)) || code;
  } catch (_) {
    return code;
  }
}

/** { code, name, german } entries for the given codes, sorted by name. */
export function countryOptions(codes = COUNTRY_CODES) {
  return [...new Set(codes.map((c) => String(c).trim().toUpperCase()).filter(Boolean))]
    .map((code) => {
      let german = '';
      try {
        german = (code === 'XK' ? 'Kosovo' : de?.of(code)) || '';
      } catch (_) {}
      return { code, name: countryName(code), german };
    })
    .sort((a, b) => a.name.localeCompare(b.name));
}

/**
 * Countries matching a typed name (English or German) or code: exact code first, then names
 * starting with the text, then names containing it.
 */
export function searchCountries(options, query) {
  const q = query.trim().toLowerCase();
  if (!q) return [];
  const rank = (c) => {
    const names = [c.name.toLowerCase(), c.german.toLowerCase()];
    if (c.code.toLowerCase() === q) return 0;
    if (names.some((n) => n.startsWith(q))) return 1;
    if (names.some((n) => n.includes(q))) return 2;
    return -1;
  };
  return options
    .map((c) => ({ c, r: rank(c) }))
    .filter((x) => x.r >= 0)
    .sort((a, b) => a.r - b.r)
    .map((x) => x.c);
}

/**
 * Countries the shop ships to, from the public shipping providers: the countries listed in zones
 * with rates, or every country when a default zone with rates exists (or nothing is configured,
 * in which case the backend does not restrict the destination either).
 */
export function shippingCountryCodes(providers = []) {
  const zones = providers.flatMap((p) => p.zones || []).filter((z) => (z.rates || []).length > 0);
  if (zones.length === 0 || zones.some((z) => z.is_default)) return COUNTRY_CODES;
  return zones.flatMap((z) => (Array.isArray(z.country_codes) ? z.country_codes : []));
}
