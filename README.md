# Personal Journal/blog site V1

## Frontend
The frontend will consist of `HTML`, `HTMX`, and `SCSS`.  `Javascript` will be used as a last resort and if needed,  it will be `Typescript`.  The development platform will be `Firefox`, my IDE will be `emacs`, and the template engine will be a `Jinja2` clone called `Askama`.

## Backend
The backend will be written in `Rust`.  The `Actix-web` is the web server.  This is the first web application I will be making from a repostitory template.  The template gives options to use any of four database singularly or in tandem. The databases are `Redis` and `Mongodb`.

## Business Logic

The design and business logic of the site are to be a blog type of personal log entries.  I fast several times a year, amongst other notable life events.  Thus, I would like to log events in my life that is accessible over the internet.


![Hand sketched design](./static/imgs/first_iter_SLS_site.jpeg)


## TODO
- [ ] insert a user login (The app will be available over the internet)
- [ ] implement user authorization 
- [X] implement user athentication
- [X] Mongodb to save user information
- [X] Hash and salt user passwords
- [X] Write tests to ensure functionality user authentication
- [ ] Write tests to ensure functionality user authorization
- [ ] Setup Github Actions to function on push
- [ ] Implement the mobile view of the web application
- [X] Setup Redis as a middle layer for near realtime retrieval
- [ ] Deploy the site to be self-hosted on the internet via a raspberry pi 4
