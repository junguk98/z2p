## zero2prod

### requirements

```yaml
# ./configuration/local.yaml
application:
  base_url: "http://127.0.0.1:8000"
  host: 127.0.0.1
database:
  require_ssl: false
email_client:
  base_url: "https://your-email-service.where"
  sender_email: "sender@here.zz"
  authorization_token: "your-token-here"
  timeout_milliseconds: 10000
```

### local

prepare for docker

use `chef` for caching docker image

```shell
$  cargo chef prepare --recipe-path recipe.json 
```

```shell
$ ./scripts/init_db.sh
$ cargo run
```

### test

```shell
$ cargo test
```

```shell
# You can delete databases that were used for testing purposes.
$ ./scripts/drop_test_db.sh {{CONTAINER_NAME}}
```


