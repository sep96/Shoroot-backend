    mod config;
    mod db;
    mod auth;
    mod bets;
    mod users;
    mod models;
    use config::Config;
    use db::connect;
    use users::handler::{register_user, login_user,get_user_balance};
    use actix_web::{App, HttpServer, web};
    use bets::handler::{place_bet , get_user_bets , submit_event_result };
    use utoipa::OpenApi;  
    use utoipa_swagger_ui::SwaggerUi; 

   #[derive(OpenApi)]
   #[openapi(
        paths(
            register_user,
            login_user,
            get_user_balance,
            place_bet,
            get_user_bets,
            submit_event_result
        ),
        components(
            schemas(User, RegisterInput, LoginInput, PlaceBetInput, EventResultInput)
        ),
        tags(
            (name = "Betting API", description = "Endpoints for sports betting")
        )
    )]
    pub struct ApiDoc;


    #[actix_web::main]
    async fn main() -> std::io::Result<()> {
        dotenv::dotenv().ok();
        let config = Config::from_env();
        let pool = connect(&config.database_url).await;

        println!("Server running at http://127.0.0.1:8080");

        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(register_user)
                .service(login_user)
                .service(get_user_balance)
                .service(place_bet)
                .service(get_user_bets)
                .service(submit_event_result)
                .service(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()))
        })
        .bind("127.0.0.1:8080")?
        .run()
        .await
    }
