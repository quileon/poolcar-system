#!/usr/bin/env fish
# Script to build the poolcar_system.
# Build both the frontend and backend
# then push the image to Gitea registry

set FRONTEND_DIR "frontend"
set BACKEND_DIR  "backend"
set IMAGE_NAME   "poolcar-system"
set REPOSITORY   "gitea.quileon.me"
set USER         "quileon"

if not which tomlq &> /dev/null
    echo "tomlq is not installed"
    exit 1
else if not which jq &> /dev/null
    echo "jq is not installed"
    exit 1
else if not which docker &> /dev/null
    echo "docker is not installed"
    exit 1
end

set BACKEND_VERSION (tomlq < $BACKEND_DIR/Cargo.toml ".package.version" | string replace -a '"' '')
set FRONTEND_VERSION (jq -r '.version' $FRONTEND_DIR/package.json)
set BACKEND_TAG $BACKEND_VERSION-backend
set FRONTEND_TAG $FRONTEND_VERSION-frontend

docker build -t $REPOSITORY/$USER/$IMAGE_NAME:$BACKEND_TAG -t $REPOSITORY/$USER/$IMAGE_NAME:latest-backend -f $BACKEND_DIR/Dockerfile $BACKEND_DIR
docker push $REPOSITORY/$USER/$IMAGE_NAME:$BACKEND_TAG
docker push $REPOSITORY/$USER/$IMAGE_NAME:latest-backend

# docker build -t $REPOSITORY/$USER/$IMAGE_NAME:$FRONTEND_TAG -t $REPOSITORY/$USER/$IMAGE_NAME:latest-backend -f $FRONTEND_DIR/Dockerfile $FRONTEND_DIR
# docker push $REPOSITORY/$USER/$IMAGE_NAME:$FRONTEND_TAG
# docker push $REPOSITORY/$USER/$IMAGE_NAME:latest-frontend
