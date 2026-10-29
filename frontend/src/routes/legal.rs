use yew::prelude::*;

#[function_component(Terms)]
pub fn terms() -> Html {
    html! {
        <div class="max-w-3xl space-y-6">
            <h1 class="title">{ "Terms of use" }</h1>
            <p class="muted">{ "Last updated: October 2026" }</p>

            <section class="space-y-2">
                <h2 class="title-sm">{ "Overview" }</h2>
                <p>
                    { "jheff.media is a personal media hub operated by Jonathan Heffernan (@jheffmedia). By using this site you agree to these terms. This site is separate from Animal Garage and Stagea Stuff." }
                </p>
            </section>

            <section class="space-y-2">
                <h2 class="title-sm">{ "Content & media" }</h2>
                <p>
                    { "Photos, videos, and other media displayed on this site remain the property of their respective owners. You may not download, reproduce, or redistribute content without permission." }
                </p>
            </section>

            <section class="space-y-2">
                <h2 class="title-sm">{ "Services" }</h2>
                <p>
                    { "Track instruction, media production, race data engineering, and related services are offered on a case-by-case basis. Bookings and deliverables are subject to separate agreements." }
                </p>
            </section>

            <section class="space-y-2">
                <h2 class="title-sm">{ "Accounts" }</h2>
                <p>
                    { "You are responsible for maintaining the security of your account credentials. We may suspend accounts that abuse the service." }
                </p>
            </section>

            <section class="space-y-2">
                <h2 class="title-sm">{ "Limitation of liability" }</h2>
                <p>
                    { "This site is provided as-is. We are not liable for damages arising from use of the site or third-party embeds (scheduling, booking, social links)." }
                </p>
            </section>
        </div>
    }
}

#[function_component(Privacy)]
pub fn privacy() -> Html {
    html! {
        <div class="max-w-3xl space-y-6">
            <h1 class="title">{ "Privacy policy" }</h1>
            <p class="muted">{ "Last updated: October 2026" }</p>

            <section class="space-y-2">
                <h2 class="title-sm">{ "What we collect" }</h2>
                <p>
                    { "Accounts store email, username, and a hashed password; when you enroll in 2FA we also store the authenticator secret. Contact and booking forms store your submission (including kind). Signed-in print flow stores photo selections and unpaid print requests. The server keeps access logs." }
                </p>
            </section>

            <section class="space-y-2">
                <h2 class="title-sm">{ "How we use data" }</h2>
                <p>
                    { "Account data is for authentication and account features. Contact and booking submissions are used to reply about projects and availability. Print selections and unpaid requests are used to fulfill print orders. Access logs support security and operations and stay on the server." }
                </p>
            </section>

            <section class="space-y-2">
                <h2 class="title-sm">{ "Third parties" }</h2>
                <p>
                    { "Optional calendar or social embeds may load from external providers with their own policies. The site loads Inter from Google Fonts (fonts.googleapis.com). No third-party analytics or telemetry product is installed." }
                </p>
            </section>

            <section class="space-y-2">
                <h2 class="title-sm">{ "Your rights" }</h2>
                <p>
                    { "Request deletion of your account or contact/booking submissions by emailing the address on the contact page. There is no self-serve delete UI." }
                </p>
            </section>
        </div>
    }
}
