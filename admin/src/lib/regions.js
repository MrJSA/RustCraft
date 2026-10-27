// World regions for shipping zones, built from the UN M49 geographic regions, plus presets that
// follow the zone structure of DHL's international parcel price list (EU, UK & Switzerland,
// rest of Europe, Russia, Americas, North Africa & Middle East, Asia & Africa without them,
// Australia & Oceania). Carriers change their country lists from time to time — check the
// current assignment with the carrier (DHL: dhl.de/laenderzonen) before setting prices.

import { EU_COUNTRIES } from '$lib/countries.js';

const list = (s) => s.split(' ');

// UN M49 sub-regions
const NORTHERN_AFRICA = list('DZ EG LY MA SD TN EH');
const SUB_SAHARAN_AFRICA = list(
  // Eastern, Middle, Southern and Western Africa
  'BI KM DJ ER ET KE MG MW MU YT MZ RE RW SC SO SS UG TZ ZM ZW IO ' +
    'AO CM CF TD CG CD GQ GA ST ' +
    'BW SZ LS NA ZA ' +
    'BJ BF CV CI GM GH GN GW LR ML MR NE NG SH SN SL TG'
);
const NORTHERN_AMERICA = list('US CA BM GL PM');
const CENTRAL_AMERICA = list('MX BZ CR SV GT HN NI PA');
const CARIBBEAN = list('AI AG AW BS BB BQ VG KY CU CW DM DO GD GP HT JM MQ MS PR BL KN LC MF VC SX TT TC VI');
const SOUTH_AMERICA = list('AR BO BR CL CO EC FK GF GY PY PE GS SR UY VE');
const CENTRAL_ASIA = list('KZ KG TJ TM UZ');
const EASTERN_ASIA = list('CN HK MO KP JP MN KR TW');
const SOUTH_EASTERN_ASIA = list('BN KH ID LA MY MM PH SG TH TL VN');
const SOUTHERN_ASIA = list('AF BD BT IN MV NP PK LK IR');
const CAUCASUS = list('AM AZ GE');
// Common definition of the Middle East (Türkiye and Cyprus are treated as Europe here)
const MIDDLE_EAST = list('BH IR IQ IL JO KW LB OM PS QA SA SY AE YE');
const EUROPE = list(
  'AT BE BG HR CY CZ DK EE FI FR DE GR HU IE IT LV LT LU MT NL PL PT RO SK SI ES SE ' +
    'AD AL AX BA BY CH FO GB GG GI IM IS JE LI MC MD ME MK NO RS RU SJ SM TR UA VA XK'
);
const OCEANIA = list('AU NZ NF CX CC FJ NC PG SB VU GU KI MH FM NR MP PW UM AS CK PF NU PN WS TK TO TV WF');

const minus = (a, b) => a.filter((c) => !b.includes(c));
const union = (...lists) => [...new Set(lists.flat())];

const AFRICA = union(NORTHERN_AFRICA, SUB_SAHARAN_AFRICA);
const ASIA = union(CENTRAL_ASIA, EASTERN_ASIA, SOUTH_EASTERN_ASIA, SOUTHERN_ASIA, CAUCASUS, MIDDLE_EAST);
const AMERICAS = union(NORTHERN_AMERICA, CENTRAL_AMERICA, CARIBBEAN, SOUTH_AMERICA);
const EUROPE_NON_EU = minus(EUROPE, EU_COUNTRIES);

export const REGIONS = [
  { group: 'Europe', id: 'eu', label: 'European Union (EU)', codes: EU_COUNTRIES },
  { group: 'Europe', id: 'europe', label: 'Europe (all)', codes: EUROPE },
  { group: 'Europe', id: 'europe-non-eu', label: 'Europe outside the EU', codes: EUROPE_NON_EU },
  { group: 'Europe', id: 'dach', label: 'DACH (Germany, Austria, Switzerland)', codes: list('DE AT CH') },
  { group: 'Americas', id: 'americas', label: 'Americas (all)', codes: AMERICAS },
  { group: 'Americas', id: 'north-america', label: 'North America', codes: NORTHERN_AMERICA },
  { group: 'Americas', id: 'central-america', label: 'Central America (incl. Mexico)', codes: CENTRAL_AMERICA },
  { group: 'Americas', id: 'caribbean', label: 'Caribbean', codes: CARIBBEAN },
  { group: 'Americas', id: 'south-america', label: 'South America', codes: SOUTH_AMERICA },
  { group: 'Africa', id: 'africa', label: 'Africa (all)', codes: AFRICA },
  { group: 'Africa', id: 'north-africa', label: 'North Africa', codes: NORTHERN_AFRICA },
  { group: 'Africa', id: 'sub-saharan-africa', label: 'Africa without North Africa', codes: SUB_SAHARAN_AFRICA },
  { group: 'Asia', id: 'asia', label: 'Asia (all, incl. Middle East)', codes: ASIA },
  { group: 'Asia', id: 'middle-east', label: 'Middle East', codes: MIDDLE_EAST },
  { group: 'Asia', id: 'asia-without-middle-east', label: 'Asia without Middle East', codes: minus(ASIA, MIDDLE_EAST) },
  { group: 'Asia', id: 'east-asia', label: 'East Asia', codes: EASTERN_ASIA },
  { group: 'Asia', id: 'south-east-asia', label: 'South-East Asia', codes: SOUTH_EASTERN_ASIA },
  { group: 'Asia', id: 'south-asia', label: 'South Asia', codes: SOUTHERN_ASIA },
  { group: 'Asia', id: 'central-asia', label: 'Central Asia', codes: CENTRAL_ASIA },
  { group: 'Oceania', id: 'australia-nz', label: 'Australia & New Zealand', codes: list('AU NZ') },
  { group: 'Oceania', id: 'oceania', label: 'Australia & Oceania', codes: OCEANIA },
  // Zone structure of DHL's international parcel prices
  { group: 'DHL zones', id: 'dhl-eu', label: 'DHL Zone EU: European Union', codes: EU_COUNTRIES },
  { group: 'DHL zones', id: 'dhl-2', label: 'DHL Zone 2: United Kingdom & Switzerland', codes: list('GB CH') },
  { group: 'DHL zones', id: 'dhl-3', label: 'DHL Zone 3: Rest of Europe', codes: minus(EUROPE_NON_EU, list('GB CH RU')) },
  { group: 'DHL zones', id: 'dhl-4', label: 'DHL Zone 4: Russia', codes: list('RU') },
  { group: 'DHL zones', id: 'dhl-5', label: 'DHL Zone 5: North, Central & South America', codes: AMERICAS },
  { group: 'DHL zones', id: 'dhl-6', label: 'DHL Zone 6: North Africa & Middle East', codes: union(NORTHERN_AFRICA, MIDDLE_EAST) },
  {
    group: 'DHL zones',
    id: 'dhl-7',
    label: 'DHL Zone 7: Asia & Africa (without North Africa & Middle East)',
    codes: union(minus(ASIA, MIDDLE_EAST), SUB_SAHARAN_AFRICA)
  },
  { group: 'DHL zones', id: 'dhl-8', label: 'DHL Zone 8: Australia & Oceania', codes: OCEANIA }
];

export const REGION_GROUPS = [...new Set(REGIONS.map((r) => r.group))];
