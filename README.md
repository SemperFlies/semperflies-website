
## Backup instructions (from the VPS)

> From [This helpful site](https://sqlbak.com/blog/how-to-automate-postgresql-database-backups-in-linux/)


Once you've shhed into the vps, you'll need to get into the docker container that contains the database. Do this by running:
```shell
sudo docker exec -it semperfliesDB bash
```
Once you're in the database container, run the following to signin as the postgres user:
```shell
su - postgres
```
Then run the following to backup.
```shell
pg_dump -U admin -d rust_hs256 -F c -f /var/lib/postgresql/data/db_backup.dump
```
> **TIP**: See `.env` for info on postgres database credentials
