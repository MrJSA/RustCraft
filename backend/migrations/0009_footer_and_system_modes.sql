-- Migration 0009: Footer configuration, 3-section layout, follow us social links, and supported payment icons
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS footer_config JSONB NOT NULL DEFAULT '{
  "branding_mode": "full",
  "menu_layout": "columns",
  "columns": [
    {
      "title": "Customer Service",
      "links": [
        { "label": "Shipping Policy & Rates", "url": "/policies/shipment-policy" },
        { "label": "Return Policy", "url": "/policies/return-policy" },
        { "label": "Revocation Policy & Form", "url": "/policies/revocation-policy" },
        { "label": "Track Order", "url": "/track" }
      ]
    },
    {
      "title": "Legal & Privacy",
      "links": [
        { "label": "Legal Notice (Impressum)", "url": "/policies/legal-notice" },
        { "label": "Terms and Conditions (AGB)", "url": "/policies/terms-conditions" },
        { "label": "Privacy Policy (GDPR)", "url": "/policies/privacy-policy" },
        { "label": "Cookie Policy", "url": "/policies/cookie-policy" }
      ]
    },
    {
      "title": "Store & Support",
      "links": [
        { "label": "Contact Information", "url": "/policies/contact" }
      ]
    }
  ],
  "social_links": {
    "github": "https://github.com",
    "twitter": "https://x.com",
    "instagram": "",
    "youtube": "",
    "facebook": "",
    "discord": "https://discord.gg",
    "whatsapp": ""
  },
  "enabled_socials": ["github", "twitter", "discord"],
  "show_socials": true,
  "show_payments": true,
  "copyright_format": "standard",
  "custom_copyright": ""
}'::jsonb;
