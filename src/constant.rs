macro_rules! TABLE_ROW {
    () => {
        "<tr>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>
                <button class=\"btn btn-danger\"
                hx-get=\"/expenses/{}/edit\"
                hx-trigger=\"edit\"
                _=\"on click
                   if .editing is not empty
                       Swal.fire({{title: 'Already Editing',
                                  showCancelButton: true,
                                  confirmButtonText: 'Yep, Edit This Row!',
                                  text:'Hey!  You are already editing a row!  Do you want to cancel that edit and continue?'}})
                       if the result's isConfirmed is false
                         halt
                       end
                       send cancel to .editing
                     end
                     trigger edit\">
                    Edit
                </button>
            </td>
        </tr>"
    };
}

macro_rules! EDITABLE_TABLE_ROW {
    () => {
        "<tr hx-trigger='cancel' class='editing' hx-get=\"/expenses/{}\">
            <td><input type='date' name='date' value='{}'></td>
            <td><input type='text' name='description' value='{}'></td>
            <td><input type='number' step='0.01' name='price' value='{}'></td>
            <td><select name='expense_type'>
                <option value='Food'>Food</option>
                <option value='Transport'>Transport</option>
                <option value='Health'>Health</option>
                <option value='Education'>Education</option>
                <option value='Entertainment'>Entertainment</option>
                <option value='Other'>Other</option>
            </select></td>
            <td><input type='checkbox' name='is_essencial' role='switch' value='true' {}></td>
            <td>
                <button class=\"btn btn-danger\" hx-get=\"/expenses/{}\">
                  Cancel
                </button>
                <button class=\"btn btn-danger\" hx-put=\"/expenses/{}\" hx-ext=\"json-enc\" hx-include=\"closest tr\">
                  Save
                </button>
            </td>
        </tr>"
    };
}

macro_rules! SIGN_IN_TAB {
    () => {
        "<article class=\"grid\">
            <div>
                <nav class=\"tab-list\" role=\"tablist\">
                  <ul>
                    <li>Sign In</li>
                    <li><a hx-get=\"/auth/signup\" role=\"tab\" aria-selected=\"false\" aria-controls=\"tab-content\">Sign Up</a></li>
                  </ul>
                </nav>

                <hgroup>
                <h1>Sign in</h1>
                <h2>Enter your user credentials</h2>
                </hgroup>
            </div>

            <div id=\"tab-content\" role=\"tabpanel\" class=\"tab-content\" hx-ext=\"response-targets\">
                <form id=\"signin-form\" hx-post=\"/auth/signin\" hx-ext=\"json-enc\" hx-swap=\"outerHTML\" hx-target=\"#signin-form\" hx-target-error=\"#message\">
                <input
                  type=\"text\"
                  name=\"username\"
                  placeholder=\"Username\"
                  aria-label=\"Login\"
                  autocomplete=\"nickname\"
                  required
                />
                <input
                  type=\"password\"
                  name=\"password\"
                  placeholder=\"Password\"
                  aria-label=\"Password\"
                  autocomplete=\"current-password\"
                  required
                />
                <button type=\"submit\" class=\"contrast\">Login</button>
                <fieldset>
                    <label for=\"remember\">
                        <input type=\"checkbox\" role=\"switch\" id=\"remember\" name=\"remember\" />
                        Remember me
                    </label>
                </fieldset>
                </form>
                <div id=\"message\">{}</div>
            </div>
        </article>"
    };
}

macro_rules! SIGN_UP_TAB {
    () => {
        "<article class=\"grid\">
            <div>
                <nav class=\"tab-list\" role=\"tablist\">
                  <ul>
                    <li><a hx-get=\"/auth/signin\" role=\"tab\" aria-selected=\"false\" aria-controls=\"tab-content\">Sign In</a></li>
                    <li>Sign Up</li>
                  </ul>
                </nav>

                <hgroup>
                <h1>Sign Up</h1>
                <h2>Create an account for Finnish</h2>
                </hgroup>
            </div>

            <div id=\"tab-content\" role=\"tabpanel\" class=\"tab-content\">
                <form id=\"signup-form\" hx-post=\"/auth/signup\" hx-ext=\"json-enc\" hx-swap=\"outerHTML\" hx-target=\"#signup-form\">
                <div hx-target=\"this\" hx-swap=\"outerHTML\">
                    <div class=\"grid\">
                    <label for=\"username\">Username</label>
                    <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
                    </div>
                    <input
                      type=\"text\"
                      name=\"username\"
                      placeholder=\"Username\"
                      aria-label=\"Login\"
                      autocomplete=\"nickname\"
                      pattern=\"[0-9a-z]{3,20}\"
                      title=\"3 to 20 characters, lowercase letters or numbers only\"
                      hx-get=\"/validate/username\"
                      hx-sync=\"closest form:abort\"
                      hx-indicator=\"#ind\"
                      required
                    />
                </div>
                <div hx-target=\"this\" hx-swap=\"outerHTML\">
                    <div class=\"grid\">
                    <label for=\"email\">Email</label>
                    <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
                    </div>
                    <input
                      type=\"email\"
                      name=\"email\"
                      placeholder=\"email@server.com\"
                      aria-label=\"Email\"
                      autocomplete=\"email\"
                      hx-get=\"/validate/email\"
                      hx-sync=\"closest form:abort\"
                      hx-indicator=\"#ind\"
                      required
                    />
                </div>
                <label for=\"password\">Password</label>
                <input
                  type=\"password\"
                  name=\"password\"
                  placeholder=\"Password\"
                  aria-label=\"Password\"
                  autocomplete=\"current-password\"
                  id=\"password\"
                  required
                />
                <div hx-target=\"this\" hx-swap=\"outerHTML\">
                    <div class=\"grid\">
                    <label for=\"confirm_password\">Confirm Password</label>
                    <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
                    </div>
                    <input
                      type=\"password\"
                      name=\"confirm_password\"
                      placeholder=\"Password\"
                      aria-label=\"Password\"
                      id=\"confirm_password\"
                      hx-get=\"/validate/passwords\"
                      hx-sync=\"closest form:abort\"
                      hx-indicator=\"#ind\"
                      hx-include=\"#password\"
                      required
                    />
                </div>
                <button type=\"submit\" class=\"contrast\">Sign up</button>
                </form>
            </div>
        </article>"
    };
}

macro_rules! VALID_EMAIL {
    () => {
        "<div hx-target=\"this\" hx-swap=\"outerHTML\">
            <div class=\"grid\">
            <label for=\"email\">Email</label>
            <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
            </div>
            <input
              type=\"email\"
              name=\"email\"
              placeholder=\"email@server.com\"
              aria-label=\"Email\"
              aria-invalid=\"false\"
              autocomplete=\"email\"
              hx-get=\"/validate/email\"
              hx-sync=\"closest form:abort\"
              hx-indicator=\"#ind\"
              value=\"{}\"
              required
            />
        </div>"
    };
}

macro_rules! INVALID_EMAIL {
    () => {
        "<div hx-target=\"this\" hx-swap=\"outerHTML\">
            <div class=\"grid\">
            <label for=\"email\">Email</label>
            <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
            </div>
            <input
              type=\"email\"
              name=\"email\"
              placeholder=\"email@server.com\"
              aria-label=\"Email\"
              aria-invalid=\"true\"
              autocomplete=\"email\"
              hx-get=\"/validate/email\"
              hx-sync=\"closest form:abort\"
              hx-indicator=\"#ind\"
              value=\"{}\"
              required
            />
            <div class='error-message' style=\"color:red;\">Please enter a valid email address, such as someone@gmail.com.</div>
        </div>"
    };
}

macro_rules! EMAIL_TAKEN {
    () => {
        "<div hx-target=\"this\" hx-swap=\"outerHTML\">
            <div class=\"grid\">
            <label for=\"email\">Email</label>
            <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
            </div>
            <input
              type=\"email\"
              name=\"email\"
              placeholder=\"email@server.com\"
              aria-label=\"Email\"
              aria-invalid=\"true\"
              autocomplete=\"email\"
              hx-get=\"/validate/email\"
              hx-sync=\"closest form:abort\"
              hx-indicator=\"#ind\"
              value=\"{}\"
              required
            />
            <div class='error-message' style=\"color:red;\">That email is already taken.  Please enter another email address.</div>
        </div>"
    };
}

macro_rules! VALID_USERNAME {
    () => {
        "<div hx-target=\"this\" hx-swap=\"outerHTML\">
            <div class=\"grid\">
            <label for=\"username\">Username</label>
            <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
            </div>
            <input
              type=\"text\"
              name=\"username\"
              placeholder=\"Username\"
              aria-label=\"Login\"
              aria-invalid=\"false\"
              autocomplete=\"nickname\"
              pattern=\"[0-9a-z]{{3,20}}\"
              title=\"3 to 20 characters, lowercase letters or numbers only\"
              hx-get=\"/validate/username\"
              hx-sync=\"closest form:abort\"
              hx-indicator=\"#ind\"
              value=\"{}\"
              required
            />
        </div>"
    };
}

macro_rules! INVALID_USERNAME {
    () => {
        "<div hx-target=\"this\" hx-swap=\"outerHTML\">
            <div class=\"grid\">
            <label for=\"username\">Username</label>
            <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
            </div>
            <input
              type=\"text\"
              name=\"username\"
              placeholder=\"Username\"
              aria-label=\"Login\"
              aria-invalid=\"true\"
              autocomplete=\"nickname\"
              pattern=\"[0-9a-z]{{3,20}}\"
              title=\"3 to 20 characters, lowercase letters or numbers only\"
              hx-get=\"/validate/username\"
              hx-sync=\"closest form:abort\"
              hx-indicator=\"#ind\"
              value=\"{}\"
              required
            />
            <div class='error-message' style=\"color:red;\">Username should be 3 to 20 characters long and only consist of lowercase letters or numbers.  Please enter another username.</div>
        </div>"
    };
}

macro_rules! USERNAME_TAKEN {
    () => {
        "<div hx-target=\"this\" hx-swap=\"outerHTML\">
            <div class=\"grid\">
            <label for=\"username\">Username</label>
            <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
            </div>
            <input
              type=\"text\"
              name=\"username\"
              placeholder=\"Username\"
              aria-label=\"Login\"
              aria-invalid=\"true\"
              autocomplete=\"nickname\"
              pattern=\"[0-9a-z]{{3,20}}\"
              title=\"3 to 20 characters, lowercase letters or numbers only\"
              hx-get=\"/validate/username\"
              hx-sync=\"closest form:abort\"
              hx-indicator=\"#ind\"
              value=\"{}\"
              required
            />
            <div class='error-message' style=\"color:red;\">That username is already taken.  Please enter another username.</div>
        </div>"
    };
}

macro_rules! MATCHING_PASSWORDS {
    () => {
        "<div hx-target=\"this\" hx-swap=\"outerHTML\">
            <div class=\"grid\">
            <label for=\"confirm_password\">Confirm Password</label>
            <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
            </div>
            <input
              type=\"password\"
              name=\"confirm_password\"
              placeholder=\"Password\"
              aria-label=\"Password\"
              aria-invalid=\"false\"
              id=\"confirm_password\"
              hx-get=\"/validate/passwords\"
              hx-sync=\"closest form:abort\"
              hx-indicator=\"#ind\"
              hx-include=\"#password\"
              value=\"{}\"
              required
            />
        </div>"
    };
}

macro_rules! MISMATCHING_PASSWORDS {
    () => {
        "<div hx-target=\"this\" hx-swap=\"outerHTML\">
            <div class=\"grid\">
            <label for=\"confirm_password\">Confirm Password</label>
            <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
            </div>
            <input
              type=\"password\"
              name=\"confirm_password\"
              placeholder=\"Password\"
              aria-label=\"Password\"
              aria-invalid=\"true\"
              id=\"confirm_password\"
              hx-get=\"/validate/passwords\"
              hx-sync=\"closest form:abort\"
              hx-indicator=\"#ind\"
              hx-include=\"#password\"
              value=\"{}\"
              required
            />
            <div class='error-message' style=\"color:red;\">Passwords don't match.</div>
        </div>"
    };
}

pub(crate) use EDITABLE_TABLE_ROW;
pub(crate) use EMAIL_TAKEN;
pub(crate) use INVALID_EMAIL;
pub(crate) use INVALID_USERNAME;
pub(crate) use MATCHING_PASSWORDS;
pub(crate) use MISMATCHING_PASSWORDS;
pub(crate) use SIGN_IN_TAB;
pub(crate) use SIGN_UP_TAB;
pub(crate) use TABLE_ROW;
pub(crate) use USERNAME_TAKEN;
pub(crate) use VALID_EMAIL;
pub(crate) use VALID_USERNAME;
