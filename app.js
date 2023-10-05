const express = require("express");
const path = require("path");
const ejs = require("ejs");
const bodyParser = require("body-parser");
const MongoClient = require("mongodb").MongoClient;
const mongoose = require("mongoose");
const nodeMailer = require("nodemailer");
const { app, BrowserWindow } = require("electron");
require("dotenv").config();

const app = express();

// ENV Variables
const port = process.env.PORT;
const uri = process.env.CONNECTIONSTRING;

// Connection to MongoDB Cloud
const client = new MongoClient(uri, {
  useNewUrlParser: true,
  useUnifiedTopology: true,
});
mongoose.connect(uri, { useNewUrlParser: true, useUnifiedTopology: true });
client.connect((err) => {
  if (err) {
    console.error("Connection error", err);
    return;
  }
  client.close();
});
const db = mongoose.connection;
db.on("error", console.error.bind(console, "connection error:"));
db.once("open", function () {
  console.log("Connected to MongoDB");
});
// Schema and Data's
const Schema = mongoose.Schema;
const userSchema = new Schema({
  username: String,
  email: String,
  password: String,
});
const User = mongoose.model("User", userSchema);
// Collection Variable to perform actions on collection
const collection = client.db("UserAuthenticationSystem").collection("users");

// body parser use
app.use(express.json());
app.use(express.static(path.join(__dirname, "public")));
app.use(
  bodyParser.urlencoded({
    extended: true,
  })
);

// Node Mailer Setup
var transporter = nodeMailer.createTransport({
  port: 465,
  host: "smtp.hostinger.com",
  auth: {
    user: "noreply@curiouslydeveloping.in",
    pass: process.env.PASSMAIL,
  },
});

// view engine set to EJS
app.set("view engine", "ejs");

app.get("/", (req, res) => {
  res.render("index");
});

// Login Page Setup
app.post("/login", async (req, res) => {
  const user = req.body.Username;
  const pass = req.body.Password;
  try {
    const check = await collection.findOne({ username: user });
    if (check.password === pass) {
      res.render("loginSuccess");
    } else {
      res.render("loginFailure");
    }
  } catch {
    res.render("loginFailure");
  }
});

// Signup Page Render
app.get("/signup", (req, res) => {
  res.render("signup");
});

// Posting new user Signup Page Backend
app.post("/signup", async (req, res) => {
  const user = req.body.Username;
  const mail = req.body.Email;
  const newUser = new User({
    username: user,
    email: mail,
    password: req.body.Password,
  });
  const checkuser = await collection.findOne({ username: user });
  const checkmail = await collection.findOne({ email: mail });
  if (checkuser == null) {
    if (checkmail == null) {
      newUser.save();
      res.render("afterSignUp");
    }
  } else {
    res.send("Credentials already exist");
  }
});

// Forgot Password Page Render
app.get("/forgotpass", (req, res) => {
  res.render("forgotPass");
});

// Forgot Password Backend Setup
app.post("/forgotpass", async (req, res) => {
  mail = req.body.Email;
  try {
    const check = await collection.findOne({ email: mail });
    if (check != null) {
      var mailOptions = {
        from: "noreply@curiouslydeveloping.in",
        to: mail,
        subject: "Sending Email using Node.js",
        text: "That was easy!",
      };

      transporter.sendMail(mailOptions, function (error, info) {
        if (error) {
          console.log(error);
        } else {
          console.log("Email sent: " + info.response);
        }
      });
      res.send("password change link");
    } else {
      res.send("Credentials not found!!!");
    }
  } catch {
    res.send(err);
  }
});

// Starting Server, Listening to Port
app.listen(port, () => {
  console.log(`Listening on Port: ${port}`);
});
