use crate::fairings::csrf::Token as CsrfToken;
use crate::fairings::db::DBConnection;
use crate::models::issues_reported::{Issue, NewIssue, EditedIssue};
use crate::rocket::serde::json::json;

use super::HtmlResponse;
use rocket::form::{Contextual, Form};
use rocket::http::Status;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::serde::Serialize;
use rocket_db_pools::{sqlx::Acquire, Connection};
use rocket_dyn_templates::{context, Template};

// Retrieves an issue with a given uuid and displays it on an HTML page.
#[get("/issues/<uuid>", format = "text/html")]
pub async fn get_issue( 
    mut db: Connection<DBConnection>,
    uuid: &str,
    flash: Option<FlashMessage<'_>>,
) -> HtmlResponse {
    let connection = db
        .acquire()
        .await
        .map_err(|_| Status::InternalServerError)?;
    let issue = Issue::find(connection, uuid).await.map_err(|e| e.status)?;
    #[derive(Serialize)]
    struct GetIssue {
        issue: Issue,
        flash: Option<String>,
    }
    let flash_message = flash.map(|fm| String::from(fm.message()));
    let context = GetIssue {
        issue,
        flash: flash_message,
    };
    Ok(Template::render("issues/show", &context))
}

// Retrieves all issues and displays them on an HTML page.
#[get("/issues?", format = "text/html")]
pub async fn get_issues(
    mut db: Connection<DBConnection>,
) -> HtmlResponse {
    let issues = Issue::find_all(&mut db)
        .await
        .map_err(|e| e.status)?;
    let context = context! {issues: issues};
    Ok(Template::render("issues/index", context))
}

// Retrieves all issues and displays them on an HTML page to allow admin to manage issue.
#[get("/issues/manage_tickets", format = "text/html")]
pub async fn manage_issues(
    mut db: Connection<DBConnection>,
) -> HtmlResponse {
    let issues = Issue::find_all(&mut db)
        .await
        .map_err(|e| e.status)?;
    let context = context! {issues: issues};
    Ok(Template::render("issues/manage_tickets", context))
}

// Displays a form to create a new issue on an HTML page.
#[get("/issues/new", format = "text/html")]
pub async fn new_issue(
    flash: Option<FlashMessage<'_>>,
 csrf_token: CsrfToken) -> HtmlResponse {
    let flash_string = flash
        .map(|fl| format!("{}", fl.message()))
        .unwrap_or_else(|| "".to_string());
    let context = context! {
        edit: false,
        form_url: "/issues",
        legend: "New Issue",
        flash: flash_string,
        csrf_token: csrf_token,
    };
    
    Ok(Template::render("issues/form", context))
}

// Creates a new issue from a form and stores it in the database.
#[post(
    "/issues",
    format = "application/x-www-form-urlencoded",
    data = "<issue_context>"
)]
pub async fn create_issue<'r>(
    mut db: Connection<DBConnection>,
    issue_context: Form<Contextual<'r, NewIssue<'r>>>,
    csrf_token: CsrfToken,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    // check whether there are existing tickets
    let last_ticket_no = Issue::find_last_token(&mut db)
    .await
    .map_err(|e| {
        let status = e;
        Flash::error(Redirect::to("/links/new"), status.to_string())
    })?;

    // set ticket to 1 if no tickets exist, else +=1
    let ticket_number = if last_ticket_no.is_empty() {
        1
    } else {
        last_ticket_no[last_ticket_no.len() - 1].ticket_number + 1
    };

    if issue_context.value.is_none() {
        let error_message = issue_context
            .context
            .errors()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("<br/>");
        return Err(Flash::error(Redirect::to("/issues/new"), error_message));
    }

    let new_issue = issue_context.value.as_ref().unwrap();
    csrf_token
        .verify(&new_issue.authenticity_token)
        .map_err(|_| {
            Flash::error(
                Redirect::to("/issues/new"),
                "Something went wrong when creating your ticket",
            )
        })?;
    let connection = db.acquire().await.map_err(|_| {
        Flash::error(
            Redirect::to("/issues/new"),
            "Something went wrong when creating your ticket",
        )
    })?;

    Issue::create(connection, new_issue, ticket_number).await.map_err(|_| {
        Flash::error(
            Redirect::to("/issues/new"),
            "Something went wrong when creating your ticket",
        )
    })?;

    let success_message = format!(
        "Successfully created ticket. Your ticket number is {}",
        ticket_number
    );

    
    Ok(Flash::success(
        Redirect::to(format!("/issues/new")),
        success_message
    ))
}

// Updates an existing issue in the database based on data submitted through a form.
#[get("/issues/edit/<uuid>", format = "text/html")]
pub async fn edit_issue(
    mut db: Connection<DBConnection>,
    uuid: &str,
    flash: Option<FlashMessage<'_>>,
    csrf_token: CsrfToken,
) -> HtmlResponse {
    let connection = db
        .acquire()
        .await
        .map_err(|_| Status::InternalServerError)?;
    let issue = Issue::find(connection, uuid).await.map_err(|e| e.status)?;
    #[derive(Serialize)]
        struct GetIssue {
        issue: Issue,
        flash: Option<String>,
        }

        println!("row: {:#?}", issue);
    let flash_string = flash
        .map(|fl| format!("{}", fl.message()))
        .unwrap_or_else(|| "".to_string());
    let context = context! {
        form_url: format!("/issues/{}", &issue.uuid),
        edit: true,
        legend: "Edit Issue",
        flash: flash_string,
        issue,
        csrf_token: csrf_token,
    };

    Ok(Template::render("issues/update_form", context))
}

// Updates an existing issue in the database based on data submitted through a form.
#[post(
    "/issues/<uuid>",
    format = "application/x-www-form-urlencoded",
    data = "<issue_context>"
)]
pub async fn update_issue<'r>(
    db: Connection<DBConnection>,
    uuid: &str,
    issue_context: Form<Contextual<'r, EditedIssue<'r>>>,
    csrf_token: CsrfToken,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    if issue_context.value.is_none() {
        let error_message = issue_context
            .context
            .errors()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("<br/>");
        return Err(Flash::error(
            Redirect::to(format!("/issues/edit/{}", uuid)),
            error_message,
        ));
    }

    let issue_value = issue_context.value.as_ref().unwrap();
    print!("error: {}",issue_value.method );
    match issue_value.method {
        "PUT" => put_issue(db, uuid, issue_context, csrf_token).await,
        "PATCH" => patch_issue(db, uuid, issue_context, csrf_token).await,
        _ => Err(Flash::error(
            Redirect::to(format!("/isues/edit/{}", uuid)),
            "Something went wrong when updating your ticket",
        )),
    }
}

// Updates an existing issue in the database based on data submitted through a form.
#[put(
    "/issues/<uuid>",
    format = "application/x-www-form-urlencoded",
    data = "<issue_context>"
)]
pub async fn put_issue<'r>(
    mut db: Connection<DBConnection>,
    uuid: &str,
    issue_context: Form<Contextual<'r, EditedIssue<'r>>>,
    csrf_token: CsrfToken,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let issue_value = issue_context.value.as_ref().unwrap();
    csrf_token
        .verify(&issue_value.authenticity_token)
        .map_err(|_| {
            Flash::error(
                Redirect::to(format!("/issues/edit/{}", uuid)),
                "Something went wrong when updating your ticket1",
            )
        })?;
        
    let issue = Issue::update(&mut db, uuid, issue_value).await.map_err(|_| {
                Flash::error(
            Redirect::to(format!("/issues/edit/{}", uuid)),
            "Something went wrong when updating your ticket2",
        )
    })?;
    Ok(Flash::success(
        Redirect::to(format!("/issues/{}", issue.uuid)),
        "Successfully updated issue",
    ))
}

// Updates an existing issue in the database based on data submitted through a form.
#[patch(
    "/issues/<uuid>",
    format = "application/x-www-form-urlencoded",
    data = "<issue_context>"
)]
pub async fn patch_issue<'r>(
    db: Connection<DBConnection>,
    uuid: &str,
    issue_context: Form<Contextual<'r, EditedIssue<'r>>>,
    csrf_token: CsrfToken,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    put_issue(db, uuid, issue_context, csrf_token).await
}

// Function to delete an issue from database
#[post("/issues/delete/<uuid>", format = "application/x-www-form-urlencoded")]
pub async fn delete_issue_entry_point(
    db: Connection<DBConnection>,
    uuid: &str,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    delete_issue(db, uuid).await
}

// Function to delete an issue from database
#[delete("/issues/<uuid>", format = "application/x-www-form-urlencoded")]
pub async fn delete_issue(
    mut db: Connection<DBConnection>,
    uuid: &str,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let connection = db.acquire().await.map_err(|_| {
        Flash::error(
            Redirect::to("/issues"),
            "Something went wrong when deleting issue",
        )
    })?;
    Issue::destroy(connection, uuid).await.map_err(|_| {
        Flash::error(
            Redirect::to("/issues/manage_tickets"),
            "Something went wrong when deleting issue",
        )
    })?;
Ok(Flash::success(
    Redirect::to("/issues/manage_tickets"),
    "Successfully deleted issue",
))

}

// Function to retrieve all open issues
#[get("/issues/open?", format = "text/html")]
pub async fn get_open(mut db: Connection<DBConnection>) -> HtmlResponse {
    let tickets = Issue::find_all(&mut db)
        .await
        .map_err(|e| e.status)?;
    println!("open tickets: {:#?}",tickets  );
    
    // Filter tickets where is_open is true
    let open_tickets = tickets.iter().filter(|t| t.status == "open").collect::<Vec<_>>();

    
    let context = context! {
        issues: open_tickets.iter().map(|t| {
            json!({
                "uuid": t.uuid,
                "issue_name": t.issue_name,
                "description": t.description,
                "reported_by": t.reported_by,
                "company_name": t.company_name,
                "contact_number": t.contact_number,
                "ticket_number": t.ticket_number,
                "ticket_owner": t.ticket_owner,
                "status": t.status,
                "created_at": t.created_at,
                "updated_at": t. updated_at,
                // add other fields as necessary
            })
        }).collect::<Vec<_>>(),
    };
    println!("open tickets: {:#?}",context  );
    
    Ok(Template::render("issues/open", context))
}


// Function to mark issue as complete
#[post(
    "/issues/complete/<uuid>",
    format = "application/x-www-form-urlencoded",
)]
pub async fn complete(
    mut db: Connection<DBConnection>,
    uuid: &str,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let connection = db.acquire().await.map_err(|_| {
        Flash::error(
            Redirect::to("/issues/complete"),
            "Something went wrong when completing issue",
        )
    })?;
    Issue::complete(connection, uuid).await.map_err(|_| {
        Flash::error(
            Redirect::to("/issues/complete"),
            "Something went wrong when completing issue",
        )
    })?;
    Ok(Flash::success(
        Redirect::to("/issues/open"),
        "Successfully completing issue",
    ))
}