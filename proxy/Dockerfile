FROM nginx:stable

COPY ./ /etc/nginx/

RUN apt-get update && apt-get upgrade -y

CMD ["/usr/sbin/nginx", "-g", "daemon off;"]