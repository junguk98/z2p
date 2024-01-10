#!/usr/bin/env bash
set -x

DB_TO_EXCLUDE=('newsletter' 'postgres')
DATABASES=()

# 데이터베이스 목록을 배열로 저장
while IFS= read -r db; do
    DATABASES+=("$db")
done < <(echo "SELECT datname FROM pg_database;" | docker exec -i postgres_1704852333 psql -U postgres -q -t)

# 배열을 이용한 for 루프
for db in "${DATABASES[@]}"; do
    db=$(echo "$db" | tr -d '[:space:]')
    if [[ ! " ${DB_TO_EXCLUDE[@]} " =~ " ${db} " ]]; then
        echo "Dropping database: ${db}"
        docker exec -t postgres_1704852333 psql -U postgres -c "DROP DATABASE IF EXISTS \"$db\";"
    else
        echo "Skipping database: ${db}"
    fi
done
