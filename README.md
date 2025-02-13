## Backup instructions (from the VPS)
> From [This helpful site](https://sqlbak.com/blog/how-to-automate-postgresql-database-backups-in-linux/)
  And [this stackoverflow](https://stackoverflow.com/questions/24718706/backup-restore-a-dockerized-postgresql-database)


Once you've shhed into the vps, you'll need to get into the docker container that contains the database. Do this by running:
(This is in `backup_db.sh`, which should be added as a cron job)
```shell
docker exec -t semperfliesDB pg_dumpall -c -U admin > backups/dump_`date +%Y-%m-%d"_"%H_%M_%S`.sql
```

To restore from a backup:
```shell
cat path/to/dump | docker exec -i semperfliesDB psql -U admin
```
> **TIP**: See `.env` for info on postgres database credentials

## Pulling changes for the website
When changes are made to the website code, a few steps need to be taken to pull those changes into the docker container without interfering with the database.


> **TIP**: If you ever need to access the image: `sudo docker exec -it semperflies-website-semperflies-1 /bin/bash
`

First, backup images in the container
```shell
sudo docker cp semperflies-website-semperflies-1:/usr/src/app/public/assets/images ~/semperflies_backups/imgs_tmp
```
Then it should be safe to stop the container
```shell
sudo docker compose stop semperflies
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
sudo docker cp ~/semperflies_backups/imgs_tmp/images semperflies-website-semperflies-1:/usr/src/app/public/assets/images
# remove the images after this
rm -rf ~/semperflies_backups/imgs_tmp/*
```

