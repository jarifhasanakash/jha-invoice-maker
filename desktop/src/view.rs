//! Builds what is printed on the invoice from a stored invoice and the
//! issuer profile. Shared by the desktop app and the web PDF engine.

use crate::dates;
use crate::layout::InvoiceView;
use crate::model::{Invoice, Profile};

pub fn view_for(p: &Profile, inv: &Invoice) -> InvoiceView {
    let custom = p.has_custom_details();
    let full = p.full_name();
    let from_name = if full.is_empty() { "Jarif Hasan".to_string() } else { full.clone() };
    let (from_location, from_email) = if custom {
        (p.address.clone(), p.email.clone())
    } else {
        ("Dhaka, Bangladesh".to_string(), "jarif.hasan@example.com".to_string())
    };
    let mobile = if !p.mobile.is_empty() {
        p.mobile.clone()
    } else if custom {
        String::new()
    } else {
        "+880 1700-000000".into()
    };
    let p1 = p.platform(0);
    let p2 = p.platform(1);
    let b = &p.bank;
    InvoiceView {
        number: if inv.invoice_number.trim().is_empty() {
            "0001".into()
        } else {
            inv.invoice_number.trim().to_string()
        },
        date: dates::display_str(&inv.date),
        client_name: if inv.client.name.is_empty() { "Client Name".into() } else { inv.client.name.clone() },
        to_location: inv.client.location.clone(),
        to_email: inv.client.email.clone(),
        from_name: from_name.clone(),
        from_location,
        from_email,
        items: inv.items.clone(),
        symbol: inv.symbol(),
        show_advance: inv.toggle_advance,
        show_balance: inv.toggle_balance,
        advance: inv.advance_amount,
        note: inv.note.clone(),
        platforms: [(p1.name, p1.email), (p2.name, p2.email)],
        bank_left: vec![
            ("First Name".into(), p.first_name.clone()),
            ("Last Name".into(), p.last_name.clone()),
            ("Address".into(), p.address.clone()),
            ("A/C No".into(), b.account_number.clone()),
        ],
        bank_right: vec![
            ("Bank Name".into(), b.bank_name.clone()),
            ("Branch Name".into(), b.branch_name.clone()),
            ("Branch Code".into(), b.branch_code.clone()),
            ("SWIFT Code".into(), b.swift_code.clone()),
            ("Routing No".into(), b.routing_no.clone()),
        ],
        signature_name: from_name,
        mobile,
    }
}
