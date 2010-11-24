// Copyright 2010 Filip Kunc. All rights reserved.

var playerImage = new Image();
playerImage.src = "Images/ball.png";

var jumpImage = new Image();
jumpImage.src = "Images/speed.png";

const tolerance = 3.0;
const maxSpeed = 5.8;
const speedPowerUp = 1.5;
const upSpeed = 7.0;
const maxFallSpeed = -15.0;
const acceleration = 1.1;
const deceleration = 1.1 * 0.2;
const changeDirectionSpeed = 3.0;
const maxSpeedUpCount = 60 * 6; // 60 FPS * 6 sec

function FPPlayer()
{
    this.x = 480.0 / 2.0 - 32.0 / 2.0;
    this.y = 320.0 / 2.0 - 32.0 / 2.0;
    this.moveX = 0.0;
    this.moveY = 0.0;
    this.rotation = 0.0;
    this.jumping = false;
    this.speedUpCounter = 0;
    this.alpha = 1.0;
    this.isVisible = true;
    
    this.rect = function()
    {
        return new FPRect(this.x, this.y, 32.0, 32.0);
    }
    
    this.move = function(offsetX, offsetY)
    {
        this.x += offsetX;
        this.y += offsetY;
    }
    
    this.update = function(game)
    {
        var inputAcceleration = game.inputAcceleration;
    	var moveLeftOrRight = false;
    	
    	if (this.speedUpCounter > 0)
    	{
    	    if (++this.speedUpCounter > maxSpeedUpCount)
    	    {
    	        this.speedUpCounter = 0;
	        }
	    }
	    
	    var currentMaxSpeed = this.speedUpCounter > 0 ? maxSpeed * speedPowerUp : maxSpeed;
	    
    	if (inputAcceleration.x < 0.0)
    	{
    		if (this.moveX < 0.0)
    			this.moveX += Math.abs(inputAcceleration.x) * acceleration * changeDirectionSpeed;
    		if (this.moveX < maxSpeed)
    			this.moveX += Math.abs(inputAcceleration.x) * acceleration;
    		moveLeftOrRight = true;
    	}
    	else if (inputAcceleration.x > 0.0)
    	{
    		if (this.moveX > 0.0)
    			this.moveX -= Math.abs(inputAcceleration.x) * acceleration * changeDirectionSpeed;
    		if (this.moveX > -maxSpeed)
    			this.moveX -= Math.abs(inputAcceleration.x) * acceleration;
    		moveLeftOrRight = true;
    	}
    	if (!this.jumping && inputAcceleration.y > 0.0)
    	{
    		if (this.moveY < upSpeed)
    			this.moveY = upSpeed;
    		this.jumping = true;
    	}

    	if (!moveLeftOrRight)
    	{
    		if (Math.abs(this.moveX) < deceleration)
    			this.moveX = 0.0;
    		else if (this.moveX > 0.0)
    			this.moveX -= deceleration;
    		else if (this.moveX < 0.0)
    			this.moveX += deceleration;
    	}	

    	this.moveY -= deceleration;
    	if (this.moveY < maxFallSpeed)
    		this.moveY = maxFallSpeed;
    	this.jumping = true;
    	
    	game.moveWorld(this.moveX, 0.0);
    	if (this.collisionLeftRight(game))
    		this.moveX = 0.0;
    	game.moveWorld(0.0, this.moveY);
    	this.collisionUpDown(game);
    	this.rotation -= this.moveX * 3.0;
    	
    	this.alpha += 0.07;
    	if (this.alpha > 3.1415926535)
    	    this.alpha -= 3.1415926535;
    }
    
    this.collisionLeftRight = function(game)
    {
    	var isColliding = false;

    	for (i in game.gameObjects)
    	{
    	    var platform = game.gameObjects[i];
    	    if (platform.isPlatform())
    		{
    			var intersection = FPRectIntersection(platform.rect(), this.rect());
    			if (intersection.isEmptyWithTolerance())
    			    continue;

    			if (platform.rect().left() > this.rect().left())
    			{
    			    if (platform.isMovable())
    			    {
    			        platform.move(intersection.size.width, 0.0);
    			        if (platform.collisionLeftRight(game))
    			        {
    			            platform.move(-intersection.size.width, 0.0);
    			            game.moveWorld(intersection.size.width, 0.0);
    			            isColliding = true;
			            }
			        }
			        else
			        {
			            game.moveWorld(intersection.size.width, 0.0);
			            isColliding = true;
		            }
    			}
    			else if (platform.rect().right() < this.rect().right())
    			{
    				if (platform.isMovable())
    			    {
    			        platform.move(-intersection.size.width, 0.0);
    			        if (platform.collisionLeftRight(game))
    			        {
    			            platform.move(intersection.size.width, 0.0);
    			            game.moveWorld(-intersection.size.width, 0.0);
    			            isColliding = true;
			            }
			        }
			        else
			        {
			            game.moveWorld(-intersection.size.width, 0.0);
			            isColliding = true;
		            }
    			}
    		}
    	}

    	return isColliding;
    }

    this.collisionUpDown = function(game)
    {
    	var isColliding = false;

    	for (i in game.gameObjects)
    	{
    	    var platform = game.gameObjects[i];
    		if (platform.isPlatform())
    		{
    			var intersection = FPRectIntersection(platform.rect(), this.rect());
    			if (intersection.isEmptyWithTolerance())
    				continue;

    			if (platform.rect().bottom() < this.rect().bottom())
    			{
    				if (this.moveY > 0.0)
    					this.moveY = 0.0;
    			
    				game.moveWorld(0.0, -intersection.size.height);
    				isColliding = true;
    			}
    			else if (this.moveY < 0.0)
    			{
    				if (platform.rect().top() > this.rect().bottom() - tolerance + this.moveY)
    				{
    					this.moveY = 0.0;
    					this.jumping = false;
    					game.moveWorld(0.0, intersection.size.height);
    					isColliding = true;
    				}
    			}
    			else if (platform.rect().top() > this.rect().bottom() - tolerance)
    			{
    				this.jumping = false;
    				game.moveWorld(0.0, intersection.size.height);
    				isColliding = true;
    			}
    		}
    	}

    	return isColliding;
    }    
    
    this.draw = function(context)
    {
        context.save();
        context.translate(240, 160);
        context.rotate(this.rotation * Math.PI / 180.0);
        context.drawImage(playerImage, -16, -16);
        context.restore();
        
        if (this.speedUpCounter > 0)
            this.drawSpeedUp(context);
    }
    
    this.drawSpeedUp = function(context)
    {
        context.globalAlpha = Math.abs(Math.sin(this.alpha)) * 0.5 + 0.5;
        context.drawImage(jumpImage, 240 - 32, 160 - 32);
        context.globalAlpha = 1.0;
    }
}