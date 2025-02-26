#![allow(unused, unreachable_code)]

use sqlx::{prelude::FromRow, Executor};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {

    let url = dotenv::var("DATABASE_URL").expect("should have .env variable DBCONNECTION");

    let pool = sqlx::PgPool::connect(&url).await?;

    let ticket = Ticket {
        ticket_id: 564652,
        customer_name: Some("jon heloooooooooooooooo".to_string()),
        customer_email: Some("jon.doe@dong.com".to_string()),
        destination: Some("Stockholm".to_string()),
        departure_country: Some("Italy".to_string()),
        ticket_price: Some(rust_decimal::Decimal::from_str_exact("123.564").unwrap()),
        ticket_datetime: Some(chrono::Utc::now()),
        ticket_type: TicketType::FirstClass,
    };

    let ticket_type = String::try_from(ticket.ticket_type).ok();

    let x = sqlx::query!(r#"insert into flight_tickets 
    (ticket_id, customer_name, customer_email, destination, departure_country, ticket_price, ticket_datetime, ticket_type) 
    VALUES($1, $2, $3, $4, $5, $6, $7, $8);"#,
    ticket.ticket_id,
    ticket.customer_name,
    ticket.customer_email,
    ticket.destination,
    ticket.departure_country,
    ticket.ticket_price,
    ticket.ticket_datetime,
    ticket_type)
    .execute(&pool)
    .await?;

    println!("{x:?}");
    let result = sqlx::query_as!(Ticket, "select * from flight_tickets where customer_name ilike '%heloooo%'")
        .fetch_all(&pool)
        .await?;

    for r in result {
        println!("{r:?}");
    }

    Ok(())
}

//ticket_id |  customer_name   |        customer_email        |  destination   | departure_country | ticket_price |        ticket_datetime        | ticket_type
//----------+------------------+------------------------------+----------------+-------------------+--------------+-------------------------------+-------------
//        1 | John Doe         | john.doe@example.com         | New York       | USA               |       500.00 | 2023-06-01 12:00:00+02        | Economy

#[derive(Debug, FromRow)]
struct Ticket {
    ticket_id: i32,
    customer_name: Option<String>,
    customer_email: Option<String>,
    destination: Option<String>,
    departure_country: Option<String>,
    ticket_price: Option<rust_decimal::Decimal>,
    ticket_datetime: Option<chrono::DateTime<chrono::Utc>>,
    ticket_type: TicketType,
}

#[derive(Debug)]
enum TicketType {
    Economy,
    Business,
    FirstClass,
    NoType,
}

impl TryFrom<TicketType> for String {
    type Error = String;

    fn try_from(value: TicketType) -> std::result::Result<Self, Self::Error> {
        match value {
            TicketType::Business => Ok("Business".to_string()),
            TicketType::Economy => Ok("Economy".to_string()),
            TicketType::FirstClass => Ok("First Class"  .to_string()),
            TicketType::NoType => Err("Notype cannot be made to string".to_string()),
        }
    }
}

impl From<Option<String>> for TicketType {
    fn from(value: Option<String>) -> Self {
        if let Some(ticket) = value {
            match ticket.as_str() {
                "Business" => TicketType::Business,
                "Economy" => TicketType::Business,
                "First Class" => TicketType::Business,
                _ => TicketType::NoType,
            }
        } else {
            TicketType::NoType
        }
    }
}
