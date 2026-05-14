#!/bin/bash
set -e

COMPOSE_DIR="$(cd "$(dirname "$0")/.." && pwd)"

show_help() {
    cat << 'HELP'
TrustRAG Log Viewer

Usage: ./logs.sh [command] [options]

Commands:
  all           Show logs from all services (default)
  backend       Show backend logs
  doc           Show doc-processor logs  
  db            Show PostgreSQL logs
  redis         Show Redis logs
  minio         Show MinIO logs
  caddy         Show Caddy logs
  errors        Show only ERROR level logs from backend
  search        Search logs for a keyword

Options:
  -f, --follow  Follow log output (live tail)
  -n NUM        Number of lines to show (default: 100)
  --since TIME  Show logs since timestamp (e.g. "1h", "2024-01-01")
  --json        Pretty-print JSON logs

Examples:
  ./logs.sh backend -f              # Follow backend logs
  ./logs.sh backend -n 500          # Last 500 lines of backend
  ./logs.sh errors --since 1h       # Errors in last hour
  ./logs.sh search "database"       # Search all logs for "database"
  ./logs.sh backend --json -n 50    # Pretty-print last 50 JSON log lines
HELP
}

CMD="${1:-all}"
shift 2>/dev/null || true

FOLLOW=""
LINES="100"
SINCE=""
JSON_MODE=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        -f|--follow) FOLLOW="--follow"; shift ;;
        -n) LINES="$2"; shift 2 ;;
        --since) SINCE="--since $2"; shift 2 ;;
        --json) JSON_MODE="1"; shift ;;
        -h|--help) show_help; exit 0 ;;
        *) shift ;;
    esac
done

service_logs() {
    local svc="$1"
    if [ "$JSON_MODE" = "1" ]; then
        docker compose -f "$COMPOSE_DIR/docker-compose.yml" logs "$svc" --tail "$LINES" $SINCE $FOLLOW 2>/dev/null | \
            while IFS= read -r line; do
                json_part="${line#*| }"
                echo "$json_part" | python3 -m json.tool 2>/dev/null || echo "$line"
            done
    else
        docker compose -f "$COMPOSE_DIR/docker-compose.yml" logs "$svc" --tail "$LINES" $SINCE $FOLLOW 2>/dev/null
    fi
}

case "$CMD" in
    all)
        docker compose -f "$COMPOSE_DIR/docker-compose.yml" logs --tail "$LINES" $SINCE $FOLLOW
        ;;
    backend)
        service_logs trustrag-backend
        ;;
    doc)
        service_logs trustrag-doc-processor
        ;;
    db)
        service_logs trustrag-postgres
        ;;
    redis)
        service_logs trustrag-redis
        ;;
    minio)
        service_logs trustrag-minio
        ;;
    caddy)
        service_logs trustrag-caddy
        ;;
    errors)
        docker compose -f "$COMPOSE_DIR/docker-compose.yml" logs trustrag-backend --tail 1000 $SINCE 2>/dev/null | \
            grep -iE '"level":"ERROR"|ERROR|error\.kind'
        ;;
    search)
        KEYWORD="$LINES"
        docker compose -f "$COMPOSE_DIR/docker-compose.yml" logs --tail 2000 $SINCE 2>/dev/null | \
            grep -i "$KEYWORD"
        ;;
    -h|--help|help)
        show_help
        ;;
    *)
        echo "Unknown command: $CMD"
        show_help
        exit 1
        ;;
esac
