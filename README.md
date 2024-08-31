# bin
a paste bin.

This is a fork of [Travus](https://github.com/Travus/)'s [bin](https://github.com/Travus/bin) project, which itself is a fork of [w4](https://github.com/w4/)'s [bin](https://github.com/w4/bin)

This fork changes 2 aspects of the Travus's bin:

##### 1. Paste IDs
This version of bin uses a 6 character long randomly generated string of upper or lower case letters.

##### 2. Custom styles
Has auto dark/light mode, updated padding, color scheme and better icons.

## Self Hosting

Easiest way is to use `docker compose`

```yml
services:
  app:
    image: ghcr.io/dpi0/bin:main
    container_name: bin
    ports:
      - "8000:8000"
    volumes:
      - pastes:/app/pastes
    user: 1000:1000

volumes:
  pastes:
```

#### Using Bind Mounts

If you prefer using bind mounts, create the `pastes` directory manually first, ensuring it has the correct `UID:GID` of the user:

```bash
mkdir -p /path/to/your/pastes
```

and then 

```yml
services:
  app:
    image: ghcr.io/dpi0/bin:main
    container_name: bin
    ports:
      - "8000:8000"
    volumes:
      - /path/to/your/pastes:/app/pastes
    user: 1000:1000
```


For more information, see [Travus](https://github.com/Travus/)'s [repository](https://github.com/Travus/bin).
