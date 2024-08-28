FROM node:slim
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable 

# Copy sources
COPY . .

# Install dependencies & build
RUN rm -rf node_modules
RUN pnpm install
RUN pnpm build

# Configure
ENV NODE_ENV=production
EXPOSE 3000

# Run
CMD ["node", "build"]
