# Semperflies site readme
This contains helpful information for backing up and updating the site on the VPS side.

> **REMEMBER**
A swapfile was used for building the project

```shell
sudo fallocate -l 2G /swapfile         # Allocate 2GB for swap
sudo chmod 600 /swapfile              # Secure it
sudo mkswap /swapfile                 # Format it as swap
sudo swapon /swapfile                 # Enable it
```

## Backup instructions (from the VPS)
> From [This helpful site](https://sqlbak.com/blog/how-to-automate-postgresql-database-backups-in-linux/)
  And [this stackoverflow](https://stackoverflow.com/questions/24718706/backup-restore-a-dockerized-postgresql-database)

If you ever need to manually backup the database, just run `backup_db.sh`

To restore from a backup:
```shell
cat path/to/dump | sudo docker exec -i semperfliesDB psql -U admin -d rust_hs256
```
> **TIP**: See `.env` for info on postgres database credentials

## Pulling changes for the website
When changes are made to the website code, a few steps need to be taken to pull those changes into the docker container without interfering with the database.

> **TIP**: If you ever need to access the image: `sudo docker exec -it semperflies-website-semperflies-1 /bin/bash
`

First, backup images in the container
```shell
# You might want to clear previously backed up images
rm -rf ~/semperflies_backups/imgs_tmp/*
sudo docker cp semperflies-website-semperflies-1:/usr/src/app/public/assets/images ~/semperflies_backups/imgs_tmp
```
Then it should be safe to stop the container
```shell
sudo docker compose stop semperflies
# then remove
sudo docker rm semperflies-website-semperflies-1
```

**NOT PART OF PULLING!**
These two steps can also be run on the database
```shell
sudo docker compose stop postgres
# then remove
sudo docker rm semperfliesDB
```

..and pull from github
```shell
git pull
```


Once the latest code is pulled, rebuild and restart the container to apply the changes. This step will not affect your database since it only restarts the application container.
```shell
sudo docker compose up -d --no-deps --build semperflies
```

to restore the backup up images:
```shell
./move_images.sh
# remove the images after this if you want
rm -rf ~/semperflies_backups/imgs_tmp/*
```


## How was this tested
I tested that both image repopulation and database backup restoration would work with the following scripts:

### Image repopuplation
After uploading an image to some place, such as dedications:
```shell
rm -rf ~/semperflies_backups/imgs_tmp/*
sudo docker cp semperflies-website-semperflies-1:/usr/src/app/public/assets/images ~/semperflies_backups/imgs_tmp
# Once the images are backed up, i removed the container completely
sudo docker compose stop semperflies
sudo docker rm semperflies-website-semperflies-1
# Completely rebuild container
sudo docker compose up -d --no-deps --build semperflies
# move the images back into the container
./move_imgs.sh
```

### DB Backup restoration
```shell
# first, i manually backup
./backup_db.sh
sudo docker compose stop postgres
sudo docker rm semperfliesDB
sudo docker volume rm semperflies-website_progresDB
# Once everything is gone i reset the database container
sudo docker compose up -d --no-deps --build postgres

# then i use the path returned by backup_db.sh to repopuplate the database
cat /home/jamie/semperflies_backups/sql/dump_2025-02-13_21-27-11.sql | sudo docker exec -i semperfliesDB psql -U admin -d rust_hs256
```


# Stripe Integration
Afer some researach, I've found that stripe allows us to host products and product information on their site. Instead of us hosting products on our database we could just use stripe to host all product info and I could simply populate our site with products from the stripe API.a


## Call
