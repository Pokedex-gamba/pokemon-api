FROM scratch

# Expose api port
EXPOSE 80/tcp

# Copy executable
COPY ./app ./app

# Start
ENTRYPOINT [ "./app" ]
