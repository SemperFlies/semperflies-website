# Semperflies site readme
This contains helpful information for backing up and updating the site on the VPS side.

## Backup instructions (from the VPS)
> From [This helpful site](https://sqlbak.com/blog/how-to-automate-postgresql-database-backups-in-linux/)
  And [this stackoverflow](https://stackoverflow.com/questions/24718706/backup-restore-a-dockerized-postgresql-database)

If you ever need to manually backup the database, just run `backup_db.sh`

To restore from a backup:
```shell
cat path/to/dump | docker exec -i semperfliesDB psql -U admin -d rust_hs256
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
./move_imgs.sh
# remove the images after this if you want
rm -rf ~/semperflies_backups/imgs_tmp/*
```

