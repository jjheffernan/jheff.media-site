# Design

What the site is today. Visual rules live in `frontend/styles/input.css`. Implementation work lives in [TODO.md](./TODO.md).

**Updated:** 2026-10-30

## Visual system

Edit `frontend/styles/input.css`. Pages use these names: `page`, `card`, `card-link`, `btn`, `btn-primary`, `btn-quiet`, `field`, `hero`, `kicker`, `title`, `muted`.

Tokens at the top of that file: `--bg`, `--fg`, `--surface`, `--accent`, `--radius-card`, `--radius-control`, `--page-width`, `--font-display`. Light and dark values are both there. Do not add a new long Tailwind string in a Rust template for a look that one of these classes can cover.

## Site map

Header: Home, Content (hub, galleries, shoots, social), Booking, Schedule, and a search field. On small screens, theme and sign-in sit in an Account dropdown, and search is the first row of the menu. Theme is the visitor's light or dark choice. Contact is in the footer, not the header.

| Route | Behavior |
|-------|----------|
| `/` | Photo feed leads. Content hub is the primary call to action |
| `/search` | Public galleries, shoots, photo titles, and upcoming dates. Drafts stay hidden |
| `/content` | One `GET /api/content/hub`: 3 social posts, 2 shoots, 2 galleries |
| `/social` | Real posts only. Empty keys show one muted line. No demo embed |
| `/galleries`, `/shoots` | Showcase, not a shop. Print select stays secondary. Covers are explicit. Images can be tagged featured |
| `/booking` | A request form, with an optional preferred time. Not the general contact form |
| `/schedule` | Month calendar of public dates is the default view. Optional embed stays under it |
| `/contact` | General questions only. Booking requests belong on `/booking` |
| `/account` | Password, 2FA, print size and unpaid total. Email change is not offered. Admins see a link to the dashboard |
| `/admin` | Sidebar for analytics, links, inbox, and content. Content holds galleries, shoots, schedule, library, and orders. Signed out opens sign-in. Any other role goes to `/404` |
| `/links` | Social link buttons. Admins edit the list on the dashboard |
| `/profile` | Signed-in welcome card. Signed-out visitors go home |
| `/legal/terms`, `/legal/privacy` | Static copy |
| `/other-sites` | Configured links |

## Decided

- Stay on Yew client-side rendering, Actix, and MongoDB.
- Contact email does not fail the visitor when SMTP is missing or the send fails.
- Print sizes are 8×10 ($25), 11×14 ($45), and 16×20 ($75). One size per request. Status stays unpaid. No card charge in the app.
- Detail pages reuse the existing print POST. Signed-out visitors see text and no new login modal.
- An empty featured Instagram URL returns 204 and the card is absent. Operator static JSON stays.
- Schedule dates are future public Mongo events. Embeds stay optional. Admins add, edit, and delete dates from the admin dashboard.
- Login and signup allow 8 attempts per IP per 10 minutes. 2FA blocks the session until the code is valid.
- Admin tools stay on `/admin`. Signed out opens sign-in. A signed-in account that is not an admin goes to `/404`.
- The homepage leads with the photo feed. The primary call to action is the content hub.
- Galleries and shoots are a showcase. Print select stays. The page is not a shop.
- Covers are chosen explicitly. Images are marked featured with a tag. No placeholder photo service.
- Booking is a request. An optional preferred time can ride with it and does not create a public date. Schedule shows a short list of public dates as event availability.
- Contact is separate from booking.
- Print payment is shelved. Invoices or another service will collect payment later. The site keeps the unpaid request.
- 2FA enrollment with the otpauth URL and secret is enough. No QR image.
- Email change is skipped. Do not offer it in the account UI.
- Admins edit the social link list, triage contact and booking messages, manage gallery and shoot media, and update print order status from the admin dashboard. Payment status is recorded only; the app does not charge cards.
- Immich and Nextcloud copies are admin-only, at most 6 files, and only into a draft gallery or shoot. Draft collections stay off the public site. Library credentials stay on the server.
