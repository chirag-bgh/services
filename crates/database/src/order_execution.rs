use {
    crate::{auction::AuctionId, OrderUid},
    bigdecimal::BigDecimal,
    sqlx::PgConnection,
};

pub async fn save(
    ex: &mut PgConnection,
    order: &OrderUid,
    auction: AuctionId,
    executed_fee: &BigDecimal,
) -> Result<(), sqlx::Error> {
    const QUERY: &str = r#"
INSERT INTO order_execution (order_uid, auction_id, reward, surplus_fee)
VALUES ($1, $2, $3, $4)
ON CONFLICT (order_uid, auction_id)
DO UPDATE SET reward = $3, surplus_fee = $4
;"#;
    sqlx::query(QUERY)
        .bind(order)
        .bind(auction)
        .bind(0.) // reward is deprecated but saved for historical analysis
        .bind(Some(executed_fee))
        .execute(ex)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use {super::*, sqlx::Connection};

    #[tokio::test]
    #[ignore]
    async fn postgres_save() {
        let mut db = PgConnection::connect("postgresql://").await.unwrap();
        let mut db = db.begin().await.unwrap();
        crate::clear_DANGER_(&mut db).await.unwrap();

        save(&mut db, &Default::default(), 0, &Default::default())
            .await
            .unwrap();
    }
}
