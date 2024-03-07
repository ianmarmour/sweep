use mail_parser::MessageParser;

pub fn fetch_inbox_top(username: String, password: String) -> imap::error::Result<Option<String>> {
    let domain = "imap.gmail.com";
    let tls = native_tls::TlsConnector::builder().build().unwrap();

    // we pass in the domain twice to check that the server's TLS
    // certificate is valid for the domain we're connecting to.
    let client = imap::connect((domain, 993), domain, &tls).unwrap();

    // the client we have here is unauthenticated.
    // to do anything useful with the e-mails, we need to log in
    let mut imap_session = client.login(username, password).map_err(|e| e.0)?;

    // we want to fetch the first email in the INBOX mailbox
    imap_session.select("INBOX")?;

    // Search for unread emails and get their UIDs
    let uids = imap_session.uid_search("UNSEEN")?;

    // Check if there are any unread emails
    if !uids.is_empty() {
        // UIDs are returned in ascending order, so the last UID is the latest unread email
        let latest_unread_uid = uids.iter().max().unwrap();

        // Fetch the email body of the latest unread email
        let messages = imap_session.uid_fetch(latest_unread_uid.to_string(), "BODY[]")?;
        let message = messages.iter().next().unwrap();

        // Here you can parse the email body or do whatever processing you need
        //println!("Latest unread email UID: {}", latest_unread_uid);
        //println!("Email body: {:?}", message.body());

        imap_session.uid_store(latest_unread_uid.to_string(), "+FLAGS (\\Seen)")?;

        let message = MessageParser::default()
            .parse(message.body().unwrap())
            .unwrap();

        let text_only = message.body_text(0).unwrap().to_string();
        return Ok(Some(text_only));
    } else {
        println!("No unread emails found.");
    }

    // be nice to the server and log out
    imap_session.logout()?;

    Ok(None)
}
