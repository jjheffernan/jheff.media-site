use yew::prelude::*;

#[function_component(Terms)]
pub fn terms() -> Html {
    html! {
        <div class="max-w-3xl space-y-6">
            <h1 class="title">{ "Terms of use" }</h1>
            <p class="muted">{ "Last updated: June 2026" }</p>

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
            <p class="muted">{ "Last updated: June 2026" }</p>

            <section class="space-y-2">
                <h2 class="title-sm">{ "What we collect" }</h2>
                <p>
                    { "If you create an account we store your email, username, and hashed password. If you submit the contact form we store your name, email, subject, and message to respond to inquiries." }
                </p>
            </section>

            <section class="space-y-2">
                <h2 class="title-sm">{ "How we use data" }</h2>
                <p>
                    { "Account data is used for authentication and future client-gallery features. Contact submissions are used solely to communicate about bookings and projects." }
                </p>
            </section>

            <section class="space-y-2">
                <h2 class="title-sm">{ "Third parties" }</h2>
                <p>
                    { "Embedded scheduling, booking, and social content may load from external providers (e.g. calendar or booking widgets). Those services have their own privacy policies." }
                </p>
            </section>

            <section class="space-y-2">
                <h2 class="title-sm">{ "Your rights" }</h2>
                <p>
                    { "You may request deletion of your account or contact submissions by emailing the address listed on the contact page." }
                </p>
            </section>
        </div>
    }
}
