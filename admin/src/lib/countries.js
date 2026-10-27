// ISO 3166-1 alpha-2 codes; names come from the browser (English + German) so both can be searched.
export const COUNTRY_CODES = (
  'AD AE AF AG AI AL AM AO AQ AR AS AT AU AW AX AZ BA BB BD BE BF BG BH BI BJ BL BM BN BO BQ BR BS BT BV BW BY BZ ' +
  'CA CC CD CF CG CH CI CK CL CM CN CO CR CU CV CW CX CY CZ DE DJ DK DM DO DZ EC EE EG EH ER ES ET FI FJ FK FM FO FR ' +
  'GA GB GD GE GF GG GH GI GL GM GN GP GQ GR GS GT GU GW GY HK HM HN HR HT HU ID IE IL IM IN IO IQ IR IS IT JE JM JO JP ' +
  'KE KG KH KI KM KN KP KR KW KY KZ LA LB LC LI LK LR LS LT LU LV LY MA MC MD ME MF MG MH MK ML MM MN MO MP MQ MR MS MT ' +
  'MU MV MW MX MY MZ NA NC NE NF NG NI NL NO NP NR NU NZ OM PA PE PF PG PH PK PL PM PN PR PS PT PW PY QA RE RO RS RU RW ' +
  'SA SB SC SD SE SG SH SI SJ SK SL SM SN SO SR SS ST SV SX SY SZ TC TD TF TG TH TJ TK TL TM TN TO TR TT TV TW TZ UA UG ' +
  'UM US UY UZ VA VC VE VG VI VN VU WF WS XK YE YT ZA ZM ZW'
).split(' ');

export const EU_COUNTRIES = 'AT BE BG CY CZ DE DK EE ES FI FR GR HR HU IE IT LT LU LV MT NL PL PT RO SE SI SK'.split(' ');

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

export const COUNTRIES = COUNTRY_CODES.map((code) => {
  let german = '';
  try {
    german = de?.of(code) || '';
  } catch (_) {}
  return { code, name: countryName(code), german };
}).sort((a, b) => a.name.localeCompare(b.name));

/** Countries matching a search for a name (English or German) or a code. */
export function searchCountries(query, exclude = []) {
  const q = query.trim().toLowerCase();
  const skip = new Set(exclude);
  return COUNTRIES.filter(
    (c) =>
      !skip.has(c.code) &&
      (!q || c.code.toLowerCase() === q || c.name.toLowerCase().includes(q) || c.german.toLowerCase().includes(q))
  );
}
