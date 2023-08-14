# cs510-final-proj

Portland State University - CS510 Rust Web Development
Student: Farani Lucero (rlucero@pdx.edu)

PROJECT NAME: Stochastic Cosmos (or Random Universe)
HOSTED AT: http://5.78.104.199 (hetzner.com)
ADMIN ACCOUNT: (username: admin@admin.com) 
GIT REPO: https://github.com/rluceropdx/cs510-final-proj


1) What was built?
A web app was built using Rust, the Axum framework, Postgresql database, and the NASA Image API.
The app allows a registered user to search for images by keyword from NASA. By default the NASA
API returns the top 100 matching results and didn't require a developer key. Thumbnails of the
100 images are displayed. The image description is truncated into a single line and displayed
below each thumbnail -- hovering will display the full description.

There is a link above each thumnail that pops up the image in full size. Clicking on the image 
or checkbox selects it. After selecting the desired images, the user can click on the "Stitch 
Selected Images" button where the web app will combine them into a single (simple) image. The
web app utilizes the Sitchy Core Library. Each "stitched" image is saved to the Postgresql 
database under each user's profile. Clicking on "Display Saved Stitches" shows all the users 
images and a button to delete any if desired.

If an administrator is signed on (using account admin@admin.com), an additional link to "Manage 
Users" is available. This will display all users (without admin permissions) and a checkbox to 
ban them as needed. A banned user attempting to login will be prevented and presented a warning 
that their account has been banned.


2) How it worked, what didn't work?
Almost all the necessary project objectives worked.
- user accounts use passwords and authentication during login (handled via JWT tokens)
- user and administrator roles where administrators can "ban" a user account
- users can save (and delete) their merged images (stored in Postgresql database)
- a cache of NASA API results, storing the following in the database:
  - keyword used to search
  - full URL used to query NASA Image Library API
  - the entire json response returned from the API
 - publishing project to Hetzer cloud servers using Docker images (http://5.78.104.199)


3) What lessons were learned?
I learned how useful Rust compilation error messages are. Typically other languages provide vague
or cryptic compilation errors and rarely do the file and line numbers correspond to the core bug
to resovle. The Rust compiler is very detailed and accurate.

I learned that storing binary files in the Postgresql database using Rust was very straight-
forward. I expected it would be a more convoluted process. 

I got alot of hands-on practice using Docker and deploying the web app to Hetzner cloud servers.
   
