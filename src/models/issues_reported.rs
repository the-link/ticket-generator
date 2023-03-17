use crate::errors::our_error::OurError;
use crate::fairings::db::DBConnection;

use super::clean_html;
use super::our_date_time::OurDateTime;
use chrono::offset::Utc;
use rocket::form::FromForm;
use rocket::serde::Serialize;
use rocket_db_pools::sqlx::{Acquire, FromRow, PgConnection};
use rocket_db_pools::Connection;
use uuid::Uuid;


// main Struct for issues reported
#[derive(Debug, FromRow, FromForm, Serialize)]
pub struct Issue {
    pub uuid: Uuid,
    pub issue_name: String,
    pub description: String,
    pub reported_by: String,
    pub company_name: String,
    pub contact_number: i64,
    pub ticket_number: i64,
    pub ticket_owner: String,
    pub status: String,
    pub created_at: OurDateTime,
    pub updated_at: OurDateTime,
}

impl Issue {
    // fn to retrieve row by uuid
    pub async fn find(connection: &mut PgConnection, uuid: &str) -> Result<Self, OurError> {
        let parsed_uuid = Uuid::parse_str(uuid).map_err(OurError::from_uuid_error)?;
        let query_str = "SELECT * FROM issues_reported WHERE uuid = $1";
        Ok(sqlx::query_as::<_, Self>(query_str)
            .bind(parsed_uuid)
            .fetch_one(connection)
            .await
            .map_err(OurError::from_sqlx_error)?)
    }

    // fn to retrieve all rows
    pub async fn find_all(
        db: &mut Connection<DBConnection>,
    ) -> Result<Vec<Self>, OurError> {
        let query_str = "SELECT * FROM issues_reported ORDER BY created_at DESC LIMIT $1";
        let connection = db.acquire().await.map_err(OurError::from_sqlx_error)?;
        let issues = sqlx::query_as::<_, Self>(query_str)
            .bind(10)
            .fetch_all(connection)
            .await
            .map_err(OurError::from_sqlx_error)?;
        Ok(issues)
    }

    // fn to retrieve last row of issues_reported, used to check last ticket number
    pub async fn find_last_token(
        db: &mut Connection<DBConnection>,
    ) -> Result<Vec<Self>, OurError> {
        let query_str = "SELECT * FROM issues_reported ORDER BY created_at DESC LIMIT $1";
        let connection = db.acquire().await.map_err(OurError::from_sqlx_error)?;
        let last_row = sqlx::query_as::<_, Self>(query_str)
            .bind(1)
            .fetch_all(connection)
            .await
            .map_err(OurError::from_sqlx_error)?;
        Ok(last_row)
    }

    // fn to create new instance of Issue strcut
    pub async fn create<'r>(
        connection: &mut PgConnection,
        new_issue: &'r NewIssue<'r>,
        ticket_number: i64,
    ) -> Result<Self, OurError> {
        // default values for new ticket
        let uuid = Uuid::new_v4();
        let ticket_owner = "orphan".to_string();
        let status = "open".to_string();

        // data provided by user input
        let issue_name = &(clean_html(new_issue.issue_name));
        let description = &(clean_html(new_issue.description));
        let reported_by = &(clean_html(new_issue.reported_by));
        let company_name = &(clean_html(new_issue.company_name));
        let contact_number = &(clean_html(new_issue.contact_number)).parse::<i64>().unwrap();
        let ticket_number = ticket_number;
        let ticket_owner = ticket_owner;
        let status = status;

        // psql query
        let query_str = r#"INSERT INTO issues_reported
(uuid, issue_name, description, reported_by, company_name, contact_number, ticket_number, ticket_owner, status )
VALUES
($1, $2, $3, $4, $5, $6, $7, $8, $9)
RETURNING *"#;
        Ok(sqlx::query_as::<_, Self>(query_str)
            .bind(uuid)
            .bind(issue_name)
            .bind(description)
            .bind(reported_by)
            .bind(company_name)
            .bind(contact_number)
            .bind(ticket_number)
            .bind(ticket_owner)
            .bind(status)
            .fetch_one(connection)
            
            .await
            .map_err(OurError::from_sqlx_error)?)
    }

    // fn to update a row
    pub async fn update<'r>(
        db: &mut Connection<DBConnection>,
        uuid: &'r str,
        issue: &'r EditedIssue<'r>,
    ) -> Result<Self, OurError> {
        // data provided by user input
        // let connection = db.acquire().await.map_err(OurError::from_sqlx_error)?;
        let now = OurDateTime(Utc::now());
        let issue_name = &(clean_html(issue.issue_name));
        let description = &(clean_html(issue.description));
        let reported_by = &(clean_html(issue.reported_by));
        let company_name = &(clean_html(issue.company_name));
        let contact_number = &(clean_html(issue.contact_number)).parse::<i64>().unwrap();
        let ticket_number = &(clean_html(issue.ticket_number)).parse::<i64>().unwrap();
        let ticket_owner = &(clean_html(issue.ticket_owner));
        let status = &(clean_html(issue.status));
        let mut set_strings = vec![
            "issue_name = $1",
            "description = $2",
            "reported_by = $3",
            "company_name = $4",
            "contact_number = $5",
            "ticket_number = $6",
            "ticket_owner = $7",
            "status = $8",
            "updated_at = $9",
        ];
        let where_string = "$10";

        // psql query
        let query_str = format!(
            r#"UPDATE issues_reported SET {} WHERE uuid = {} RETURNING *"#,
            set_strings.join(", "),
            where_string,
        );
        
        let connection = db.acquire().await.map_err(OurError::from_sqlx_error)?;
        let binded = sqlx::query_as::<_, Self>(&query_str)
            .bind(issue_name)
            .bind(description)
            .bind(reported_by)
            .bind(company_name)
            .bind(contact_number)
            .bind(ticket_number)
            .bind(ticket_owner)
            .bind(status)
            .bind(&now);

        let parsed_uuid = Uuid::parse_str(uuid).map_err(OurError::from_uuid_error)?;
        Ok(binded
            .bind(parsed_uuid)
            .fetch_one(connection)
            .await
            .map_err(OurError::from_sqlx_error)?)

             
    }

    // fn to delete row from table
    pub async fn destroy(connection: &mut PgConnection, uuid: &str) -> Result<(), OurError> {
        let parsed_uuid = Uuid::parse_str(uuid).map_err(OurError::from_uuid_error)?;
        let query_str = "DELETE FROM issues_reported WHERE uuid = $1";
        sqlx::query(query_str)
            .bind(parsed_uuid)
            .execute(connection)
            .await
            .map_err(OurError::from_sqlx_error)?;
        Ok(())
    }

    // fn to set status to complete
    pub async fn complete(connection: &mut PgConnection, uuid: &str) -> Result<(), OurError> {
        let parsed_uuid = Uuid::parse_str(uuid).map_err(OurError::from_uuid_error)?;
        let query_str = "UPDATE issues_reported SET status = 'closed' WHERE uuid = $1";
        sqlx::query(query_str)
            .bind(parsed_uuid)
            .execute(connection)
            .await
            .map_err(OurError::from_sqlx_error)?;
        Ok(())
    }
        
}

// struct for new instance of Issue struct and field validation
#[derive(Debug, FromForm)]
pub struct NewIssue<'r> {
    #[field(validate = len(1..20).or_else(msg!("issue name cannot be empty")))]
    pub issue_name: &'r str,
    #[field(validate = len(1..50).or_else(msg!("Description cannot be empty")))]
    pub description: &'r str,
    #[field(validate = len(1..20).or_else(msg!("Reported by cannot be empty")))]
    pub reported_by: &'r str,
    #[field(validate = len(1..20).or_else(msg!("Company name by cannot be empty")))]
    pub company_name: &'r str,
    #[field(validate = len(1..20).or_else(msg!("Contact number by cannot be empty")))]
    pub contact_number: &'r str,
    #[field(default = "")]
    pub authenticity_token: &'r str,
}

// struct for editing instance of Issue struct and field validation
#[derive(Debug, FromForm)]
pub struct EditedIssue<'r> {
    #[field(name = "_METHOD")]
    pub method: &'r str,
    #[field(validate = len(1..20).or_else(msg!("issue name cannot be empty")))]
    pub issue_name: &'r str,
    #[field(validate = len(1..20).or_else(msg!("Description cannot be empty")))]
    pub description: &'r str,
    #[field(validate = len(1..20).or_else(msg!("Reported by cannot be empty")))]
    pub reported_by: &'r str,
    #[field(validate = len(1..20).or_else(msg!("Company name by cannot be empty")))]
    pub company_name: &'r str,
    #[field(validate = len(1..20).or_else(msg!("Company name by cannot be empty")))]
    pub contact_number: &'r str,
    pub ticket_number: &'r str,
    pub ticket_owner: &'r str,
    pub status: &'r str,
    #[field(default = "")]
    pub authenticity_token: &'r str,
}
