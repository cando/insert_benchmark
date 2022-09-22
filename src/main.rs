use std::time::Duration;

use amiquip::{
    Connection, ConsumerMessage, ConsumerOptions, ExchangeDeclareOptions, ExchangeType, FieldTable,
    Publish, QueueDeclareOptions,
};
use mysql::prelude::*;
use mysql::*;
use uuid::Uuid;
fn main() -> std::result::Result<(), String> {
    std::thread::sleep(Duration::from_secs(5));
    let role = std::env::var("ROLE").expect("ROLE not set");
    let rabbit_connection_string =
        std::env::var("RABBIT_CONNECTION_STRING").expect("YOU MUST SET RABBIT CONNECTION_STRING");
    let db_connection_string =
        std::env::var("DB_CONNECTION_STRING").expect("DB_CONNECTION_STRING not set");

    let pool = Pool::new(db_connection_string.as_str()).map_err(format_db_error)?;
    let mut db_conn = pool.get_conn().map_err(|e| format!("{e}"))?;

    let mut rabbit_conn =
        Connection::insecure_open(rabbit_connection_string.as_str()).map_err(format_amqp_error)?;
    match role.as_str() {
        "PRODUCER" => produce(&mut db_conn, &mut rabbit_conn)?,
        "CONSUMER" => consume(&mut db_conn, &mut rabbit_conn)?,
        _ => panic!("what"),
    }
    Ok(())
}

fn produce(
    db_conn: &mut PooledConn,
    rabbit_conn: &mut Connection,
) -> std::result::Result<(), String> {
    let msg_per_second = std::env::var("MESSAGES_PER_SECOND")
        .expect("a producer must have MESSAGES_PER_SECOND")
        .parse::<u64>()
        .expect("MESSAGES_PER_SECOND must be a valid unsigned integer");
    let msg_rate = 1000000 / msg_per_second; //us between each message
    
    db_conn
        .query_drop(r"DROP TABLE IF EXISTS `event`;")
        .map_err(format_db_error)?;
    
    db_conn
        .query_drop(
            r"
        CREATE TABLE `event` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `event_type` varchar(255) NOT NULL,
            `aggregate_name` varchar(255) NOT NULL,
            `aggregate_id` varchar(255) NOT NULL,
            `insurance_id` varchar(255) DEFAULT NULL,
            `occurred_on` datetime NOT NULL,
            `payload` text NOT NULL,
            `inserted_at` datetime NOT NULL,
            `updated_at` datetime NOT NULL,
            PRIMARY KEY (`id`),
            KEY `event_aggregate_id_index` (`aggregate_id`),
            KEY `event_insurance_id_index` (`insurance_id`)
          ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
        ",
        )
        .map_err(format_db_error)?;

    let channel = rabbit_conn.open_channel(None).map_err(format_amqp_error)?;
    let exchange = channel
        .exchange_declare(
            ExchangeType::Fanout,
            "insert-benchmark-exchange",
            ExchangeDeclareOptions {
                durable: true,
                auto_delete: false,
                ..Default::default()
            },
        )
        .map_err(format_amqp_error)?;

    let message = r#"{"type":"insurance_issued","aggregate_name":"insurance","aggregate_id":"REPLACE_ME","occurred_on":"2016-11-22T14:57:48+0100","save_code":"PR1234567","quote_uuid":"abc","insurance_code":"PRP1234567","issuing_company_code":"PRP1234567","effective_date":"2016-11-22T14:57:48+0100","expiration_date":"2017-11-22T14:57:48+0100","source":"prima","save_source":"prima","coverages":[{"id":"3d9e3e72-123f-46ae-98bc-c84cd9695bda","date_from":"2016-11-22T14:57:48+0100","date_to":"2017-11-22T14:57:48+0100","price":100,"paid":true,"guarantees":[{"slug":"rca","limit":"limit_rca_6_07","deductible":"nodeductibles","price":{"net":100,"ssn":100,"tax":100,"gross":100}}],"progressive_instalment":1}],"risk_type":"new_business","is_substitution":false,"issuing_company":"great_lakes","vehicle_type":"car","plate_number":"CN333DR","payment_frequency":"yearly","contractor":{"name":"aaa","residence_province":"BB","residence_address":"ccc","residence_city":"ddd","fiscal_code_vat":"eee"},"owner":{"name":"fff","residence_province":"GG","residence_address":"hhh","residence_city":"iii","residence_zip":"24047","fiscal_code_vat":"jjj"}}"#;

    loop {
        let aggregate_id = Uuid::new_v4();
        let message_to_publish = message.replace("REPLACE_ME", &aggregate_id.to_string());
        exchange
            .publish(Publish::new(message_to_publish.as_bytes(), "insert-test"))
            .map_err(format_amqp_error)?;
        std::thread::sleep(Duration::from_micros(msg_rate));
    }
}

fn consume(
    db_conn: &mut PooledConn,
    rabbit_conn: &mut Connection,
) -> std::result::Result<(), String> {
    Ok(())
}

fn format_db_error(e: Error) -> String {
    format!("mysql {e}")
}

fn format_amqp_error(e: amiquip::Error) -> String {
    format!("Amiquip {e}")
}
