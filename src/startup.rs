use crate::endpoints::{self, health, index, login, register, templates, validate_email};
use crate::models::r2d2_mongodb::client_manager::MongoClientManager;
use crate::settings::Settings;
use actix_web::web::{self, Data};
use actix_web::{App, HttpServer, http::KeepAlive, middleware};
use r2d2::ManageConnection;
use std::net;
use std::time::Duration;
use tracing::{debug, info, instrument, warn};

pub const PARSE_COUNT: u8 = 9;

#[instrument(
    name = "Running the server",
    target = "demo_web_app",
    level = "info",
    skip(listener, settings)
)]
async fn run(
    listener: std::net::TcpListener,
    settings: Settings,
) -> Result<actix_web::dev::Server, std::io::Error> {
    let redis_pool: redis::Client = match redis::Client::open(settings.redis.uri.clone()) {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!("Unable to connect to the cache layer: {err:#?}");
            panic!("Application cannot start: {err:#?}")
        }
    };
    let redis_pool: r2d2::Pool<redis::Client> = match r2d2::Pool::builder()
        .max_size(settings.redis.pool_size)
        .connection_timeout(Duration::from_secs(
            settings.redis.pool_timeout_seconds.into(),
        ))
        .build(redis_pool)
    {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!("Unable to connect to the cache layer: {err:#?}");
            panic!("Application cannot start: {err:#?}")
        }
    };

    let mongo_pool: MongoClientManager =
        match MongoClientManager::from_uri(&settings.mongo.uri).await {
            Ok(conn) => conn,
            Err(err) => {
                tracing::error!("Unable to connect to the database: {err:#?}");
                panic!("Application cannot start: {err:#?}")
            }
        };
    // let mongo_settings = match settings::get() {
    //     Ok(settings) => settings,
    //     Err(err) => {
    //         tracing::error!("Unable to acquire database configurtation: {err:#?}");
    //         panic!("Application cannot start: {err:#?}")
    //     }
    // }
    // .mongo;

    let mongo_pool: mongodb::Client = match mongo_pool.connect() {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!("Unable to connect to the database: {err:#?}");
            panic!("Application cannot start: {err:#?}")
        }
    };
    // .database(&mongo_settings.db);

    // Connect to the MongoDB database
    let db_redis = Data::new(redis_pool);
    let db_mongo = Data::new(mongo_pool);
    // info!("Processed DB connection pool for distribution");

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::DefaultHeaders::new().add(("X-Version", env!("CARGO_PKG_VERSION")))) // Security consideration
            .app_data(db_redis.clone())
            .app_data(db_mongo.clone())
            .service(
                web::scope("/static")
                    .service(templates::favicon)
                    .service(templates::logomain)
                    .service(templates::usmc_patrolling)
                    .service(templates::stylesheet)
                    .service(templates::source_map)
                    .service(templates::htmx)
                    .service(templates::response_targets)
                    .service(templates::sse)
                    .service(templates::action_script)
                    .service(templates::prof_headshot)
                    .service(templates::spinner)
                    .service(templates::github)
                    .service(templates::linkedin),
            )
            .service(index::index)
            .service(health::health_check)
            .service(endpoints::bs_logic::about)
            .service(endpoints::bs_logic::schedule)
            .service(endpoints::bs_logic::testimonials)
            .service(endpoints::bs_logic::finances)
            .service(endpoints::bs_logic::contact)
            .service(
                web::scope("/v1")
                    .service(login::login_template)
                    .service(login::login_user)
                    .service(register::register_template)
                    .service(register::register_user)
                    .service(validate_email::validate_email),
            )
            .route("/sse", web::get().to(index::sse))
    })
    .keep_alive(KeepAlive::Os) // Keep the connection alive; OS handled
    .disable_signals() // Disable the signals to allow the OS to handle the signals
    .workers(2)
    .shutdown_timeout(3)
    .listen(listener)?
    .run();

    if settings.debug {
        warn!("Debug mode");
    } else {
        warn!("Production mode");
    }

    Ok(server)
}

pub struct Application {
    port: u16,
    server: actix_web::dev::Server,
}

impl Application {
    /// # Result
    ///  - `Ok(Application)` if the application was successfully built
    /// # Errors
    ///  - `std::io::Error` if the application could not be built
    /// # Panics
    ///  - If the application could not be built
    #[instrument(
        name = "Build Application",
        level = "info",
        target = "demo_web_app",
        skip(settings)
    )]
    pub async fn build(settings: &mut crate::settings::Settings) -> Result<Self, std::io::Error> {
        info!("Buidling the main application");

        let app_address = format!(
            "{}:{}",
            settings.application.host, settings.application.port
        );

        debug!("Binding the TCP port: {app_address}");
        let listener: net::TcpListener = net::TcpListener::bind(&app_address)?;
        let port = listener.local_addr()?.port();
        let server = run(listener, settings.clone()).await?;

        Ok(Self { port, server })
    }

    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// # Result
    ///  - `Ok(())` if the application was successfully started
    /// # Errors
    ///  - `std::io::Error` if the application could not be started
    /// # Panics
    ///  - If the application could not be started
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        info!("Running until stopped");
        self.server.await
    }
}
